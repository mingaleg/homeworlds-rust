use std::{mem::replace, num::NonZero};

use super::{Apply, OperationError};
use crate::public::{current_turn::PendingPowers, *};
use thiserror::Error;

pub enum UpdatePendingPowers {
    Set {
        power: common::Power,
        count: NonZero<u8>,
    },
    UseOne,
}

#[derive(Error, Debug)]
pub enum UpdatePendingPowersError {
    #[error("pending powers can only be set once per turn")]
    CanOnlyBeSetOnce,
    #[error("pending powers were not set before being used")]
    NotSet,
    #[error("pending powers were already exhausted")]
    AlreadyExhausted,
}

impl Apply for UpdatePendingPowers {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let mut pending_powers = replace(&mut state.pending_powers, PendingPowers::Nil);
        state.pending_powers = match self {
            UpdatePendingPowers::Set { power, count } => match pending_powers {
                PendingPowers::Nil => PendingPowers::Pending {
                    power,
                    count,
                    original_count: count,
                },
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
                PendingPowers::Nil => {
                    return Err(UpdatePendingPowersError::NotSet.into());
                }
                PendingPowers::Exhausted { .. } => {
                    return Err(UpdatePendingPowersError::AlreadyExhausted.into());
                }
            },
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::public::{board, common, current_turn};
    use std::num::NonZero;

    fn create_test_state() -> current_turn::CurrentTurnState {
        current_turn::CurrentTurnState {
            player: common::Player::First,
            current_turn_status: current_turn::CurrentTurnStatus::MakingActions,
            game_board: board::GameBoard {
                bank: board::Bank {
                    pyramids: Default::default(),
                },
                homeworld_first: board::StarSystem {
                    name: "Homeworld1".to_string(),
                    center: board::StarSystemCenter::Empty,
                    fleet_first: board::Fleet::default(),
                    fleet_second: board::Fleet::default(),
                    is_homeworld_for: Some(common::Player::First),
                },
                homeworld_second: board::StarSystem {
                    name: "Homeworld2".to_string(),
                    center: board::StarSystemCenter::Empty,
                    fleet_first: board::Fleet::default(),
                    fleet_second: board::Fleet::default(),
                    is_homeworld_for: Some(common::Player::Second),
                },
                discovered_systems: vec![],
            },
            pending_powers: current_turn::PendingPowers::Nil,
        }
    }

    #[test]
    fn test_set_pending_powers_success() {
        let mut state = create_test_state();
        let op = UpdatePendingPowers::Set {
            power: common::Power::Build,
            count: NonZero::new(3).unwrap(),
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(matches!(
            state.pending_powers,
            PendingPowers::Pending { .. }
        ));
    }

    #[test]
    fn test_set_pending_powers_can_only_be_set_once() {
        let mut state = create_test_state();
        state.pending_powers = PendingPowers::Pending {
            power: common::Power::Build,
            count: NonZero::new(2).unwrap(),
            original_count: NonZero::new(2).unwrap(),
        };

        let op = UpdatePendingPowers::Set {
            power: common::Power::Trade,
            count: NonZero::new(1).unwrap(),
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::UpdatePendingPowersError(
                UpdatePendingPowersError::CanOnlyBeSetOnce
            ))
        ));
    }

    #[test]
    fn test_use_one_decrement() {
        let mut state = create_test_state();
        state.pending_powers = PendingPowers::Pending {
            power: common::Power::Build,
            count: NonZero::new(3).unwrap(),
            original_count: NonZero::new(3).unwrap(),
        };

        let op = UpdatePendingPowers::UseOne;
        let result = op.apply(&mut state);
        assert!(result.is_ok());

        if let PendingPowers::Pending { count, .. } = state.pending_powers {
            assert_eq!(count.get(), 2);
        } else {
            panic!("Expected Pending state");
        }
    }

    #[test]
    fn test_use_one_exhausts() {
        let mut state = create_test_state();
        state.pending_powers = PendingPowers::Pending {
            power: common::Power::Build,
            count: NonZero::new(1).unwrap(),
            original_count: NonZero::new(3).unwrap(),
        };

        let op = UpdatePendingPowers::UseOne;
        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(matches!(
            state.pending_powers,
            PendingPowers::Exhausted { .. }
        ));
    }

    #[test]
    fn test_use_one_not_set() {
        let mut state = create_test_state();
        let op = UpdatePendingPowers::UseOne;

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::UpdatePendingPowersError(
                UpdatePendingPowersError::NotSet
            ))
        ));
    }

    #[test]
    fn test_use_one_already_exhausted() {
        let mut state = create_test_state();
        state.pending_powers = PendingPowers::Exhausted {
            power: common::Power::Build,
            original_count: NonZero::new(3).unwrap(),
        };

        let op = UpdatePendingPowers::UseOne;
        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::UpdatePendingPowersError(
                UpdatePendingPowersError::AlreadyExhausted
            ))
        ));
    }
}
