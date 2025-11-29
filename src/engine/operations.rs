use crate::public::{current_turn::PendingPowers, *};
use enum_dispatch::enum_dispatch;
use std::{collections::hash_map::Entry, hash::Hash, mem::replace, num::NonZero};
use thiserror::Error;

enum UpdatePendingPowers {
    Set {
        power: common::Power,
        count: NonZero<u8>,
    },
    UseOne,
}

enum UpdateOneDelta {
    AddOne,
    RemoveOne,
}

struct DiscoverSystem {
    name: String,
    center_star: board::Star,
}

struct UpdateFleet {
    star_system_name: String,
    player: common::Player,
    starship: board::Starship,
    delta: UpdateOneDelta,
}

struct UpdateBank {
    pyramid: common::Pyramid,
    delta: UpdateOneDelta,
}

struct ForgetSystem(board::StarSystem);

struct DestroyBinaryStar {
    star_system: board::StarSystem,
    star: board::BinaryStarId,
}

#[enum_dispatch]
enum BasicOperation {
    DiscoverSystem,
    UpdatePendingPowers,
    UpdateFleet,
    UpdateBank,
    // ForgetSystem,
    // DestroyBinaryStar,
}

#[derive(Error, Debug)]
enum OperationError {
    #[error("star system with name {name:?} already exists")]
    DuplicatedStarSystemName{name: String},
    #[error("cannot update pending powers")]
    UpdatePendingPowersError(#[from] UpdatePendingPowersError),
    #[error("unknown star system")]
    UnknownStarSystem,
    #[error("cannot update fleet")]
    UpdateFleetError(#[from] UpdateFleetError),
    #[error("cannot update bank")]
    UpdateBankError(#[from] UpdateBankError),
}

#[derive(Error, Debug)]
enum UpdatePendingPowersError {
    #[error("pending powers can only be set once per turn")]
    CanOnlyBeSetOnce,
    #[error("pending powers were not set before being used")]
    NotSet,
    #[error("pending powers were already exhausted")]
    AlreadyExhausted,
}

#[derive(Error, Debug)]
enum UpdateFleetError {
    #[error("cannot remove a starship from the fleet as there are none of such type")]
    NoSuchStarships,
    #[error("fleet count overflow - too many starships")]
    FleetCountOverflow,
}

#[derive(Error, Debug)]
enum UpdateBankError {
    #[error("cannot remove a pyramid from the bank as there are none of such type")]
    NoPyramidsInBank,
    #[error("bank count overflow - too many pyramids")]
    BankCountOverflow,
}

#[enum_dispatch(BasicOperation)]
trait Apply {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError>;
}

fn update_hashmap_count<K, E>(
    entry: Entry<K, NonZero<u8>>,
    delta: UpdateOneDelta,
    overflow_error: E,
    not_found_error: E,
) -> Result<(), E>
where
    K: Eq + Hash,
{
    match delta {
        UpdateOneDelta::AddOne => match entry {
            Entry::Occupied(mut e) => {
                let count = e.get_mut();
                *count = count.checked_add(1).ok_or(overflow_error)?;
            }
            Entry::Vacant(e) => {
                e.insert(NonZero::new(1).unwrap());
            }
        },
        UpdateOneDelta::RemoveOne => match entry {
            Entry::Occupied(mut entry) => {
                let count = entry.get_mut();
                if count.get() > 1 {
                    *count = unsafe { NonZero::new_unchecked(count.get() - 1) };
                } else {
                    entry.remove();
                }
            }
            Entry::Vacant(_) => return Err(not_found_error),
        },
    }
    Ok(())
}

impl Apply for DiscoverSystem {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let game_board = &mut state.game_board;
        if game_board
            .discovered_systems
            .iter()
            .any(|it| it.name == self.name)
        {
            return Err(OperationError::DuplicatedStarSystemName{name: self.name});
        }
        game_board.discovered_systems.push(board::StarSystem {
            name: self.name,
            center: board::StarSystemCenter::SingleStar(self.center_star),
            fleet_first: board::Fleet::default(),
            fleet_second: board::Fleet::default(),
        });
        Ok(())
    }
}

impl Apply for UpdatePendingPowers {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let mut pending_powers = replace(&mut state.pending_powers, PendingPowers::Nil);
        state.pending_powers = match self {
            UpdatePendingPowers::Set { power, count } => match pending_powers {
                PendingPowers::Nil => {
                    PendingPowers::Pending {
                        power,
                        count,
                        original_count: count,
                    }
                }
                _ => return Err(UpdatePendingPowersError::CanOnlyBeSetOnce.into()),
            },

            UpdatePendingPowers::UseOne => match &mut pending_powers {
                PendingPowers::Pending {
                    power,
                    count,
                    original_count,
                } => {
                    if count.get() > 1 {
                        *count = unsafe { NonZero::new_unchecked(count.get() - 1) };
                        pending_powers
                    } else {
                        PendingPowers::Exhausted {
                            power: replace(power, common::Power::Build),
                            original_count: *original_count,
                        }
                    }
                }
                PendingPowers::Nil => return Err(UpdatePendingPowersError::NotSet.into()),
                PendingPowers::Exhausted{..} => return Err(UpdatePendingPowersError::AlreadyExhausted.into()),
            },
        };
        Ok(())
    }
}

impl Apply for UpdateFleet {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let Some(star_system) = state.game_board.discovered_systems
            .iter_mut()
            .find(|it| it.name == self.star_system_name)
        else {
            return Err(OperationError::UnknownStarSystem);
        };
        let fleet = star_system.fleet_mut(self.player);
        let entry = fleet.starships.entry(self.starship);
        update_hashmap_count(
            entry,
            self.delta,
            UpdateFleetError::FleetCountOverflow,
            UpdateFleetError::NoSuchStarships,
        )?;
        Ok(())
    }
}

impl Apply for UpdateBank {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let bank = &mut state.game_board.bank;
        let entry = bank.pyramids.entry(self.pyramid);
        update_hashmap_count(
            entry,
            self.delta,
            UpdateBankError::BankCountOverflow,
            UpdateBankError::NoPyramidsInBank,
        )?;
        Ok(())
    }
}
