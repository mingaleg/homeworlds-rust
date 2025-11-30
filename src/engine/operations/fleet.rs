use super::common;
use super::{Apply, OperationError};
use crate::public::{board::Starship, common::Player, current_turn::CurrentTurnState};
use thiserror::Error;

pub struct UpdateFleet {
    pub star_system_name: String,
    pub player: Player,
    pub starship: Starship,
    pub delta: common::UpdateOneDelta,
}

#[derive(Error, Debug)]
pub enum UpdateFleetError {
    #[error("cannot remove a starship from the fleet as there are none of such type")]
    NoSuchStarships,
    #[error("fleet count overflow - too many starships")]
    FleetCountOverflow,
}

impl Apply for UpdateFleet {
    fn apply(self, state: &mut CurrentTurnState) -> Result<(), OperationError> {
        let Some(star_system) = state
            .game_board
            .discovered_systems
            .iter_mut()
            .find(|it| it.name == self.star_system_name)
        else {
            return Err(OperationError::UnknownStarSystem);
        };
        let fleet = star_system.fleet_mut(self.player);
        let entry = fleet.starships.entry(self.starship);
        super::utils::update_hashmap_count(
            entry,
            self.delta,
            UpdateFleetError::FleetCountOverflow,
            UpdateFleetError::NoSuchStarships,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::common::UpdateOneDelta;
    use super::*;
    use crate::public::{board, common, current_turn};
    use std::num::NonZero;

    fn create_test_state() -> current_turn::CurrentTurnState {
        let mut state = current_turn::CurrentTurnState {
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
        };
        state.game_board.discovered_systems.push(board::StarSystem {
            name: "Alpha".to_string(),
            center: board::StarSystemCenter::Empty,
            fleet_first: board::Fleet::default(),
            fleet_second: board::Fleet::default(),
            is_homeworld_for: None,
        });
        state
    }

    #[test]
    fn test_add_starship_to_empty_fleet() {
        let mut state = create_test_state();
        let starship = Starship(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });

        let op = UpdateFleet {
            star_system_name: "Alpha".to_string(),
            player: Player::First,
            starship,
            delta: UpdateOneDelta::AddOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(
            state.game_board.discovered_systems[0]
                .fleet_first
                .starships
                .get(&starship)
                .unwrap()
                .get(),
            1
        );
    }

    #[test]
    fn test_add_starship_increment() {
        let mut state = create_test_state();
        let starship = Starship(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });

        state.game_board.discovered_systems[0]
            .fleet_first
            .starships
            .insert(starship, NonZero::new(1).unwrap());

        let op = UpdateFleet {
            star_system_name: "Alpha".to_string(),
            player: Player::First,
            starship,
            delta: UpdateOneDelta::AddOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(
            state.game_board.discovered_systems[0]
                .fleet_first
                .starships
                .get(&starship)
                .unwrap()
                .get(),
            2
        );
    }

    #[test]
    fn test_remove_starship_decrement() {
        let mut state = create_test_state();
        let starship = Starship(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });

        state.game_board.discovered_systems[0]
            .fleet_first
            .starships
            .insert(starship, NonZero::new(2).unwrap());

        let op = UpdateFleet {
            star_system_name: "Alpha".to_string(),
            player: Player::First,
            starship,
            delta: UpdateOneDelta::RemoveOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(
            state.game_board.discovered_systems[0]
                .fleet_first
                .starships
                .get(&starship)
                .unwrap()
                .get(),
            1
        );
    }

    #[test]
    fn test_remove_starship_removes_entry() {
        let mut state = create_test_state();
        let starship = Starship(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });

        state.game_board.discovered_systems[0]
            .fleet_first
            .starships
            .insert(starship, NonZero::new(1).unwrap());

        let op = UpdateFleet {
            star_system_name: "Alpha".to_string(),
            player: Player::First,
            starship,
            delta: UpdateOneDelta::RemoveOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(
            !state.game_board.discovered_systems[0]
                .fleet_first
                .starships
                .contains_key(&starship)
        );
    }

    #[test]
    fn test_remove_nonexistent_starship() {
        let mut state = create_test_state();
        let starship = Starship(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });

        let op = UpdateFleet {
            star_system_name: "Alpha".to_string(),
            player: Player::First,
            starship,
            delta: UpdateOneDelta::RemoveOne,
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::UpdateFleetError(
                UpdateFleetError::NoSuchStarships
            ))
        ));
    }

    #[test]
    fn test_unknown_star_system() {
        let mut state = create_test_state();
        let starship = Starship(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });

        let op = UpdateFleet {
            star_system_name: "Unknown".to_string(),
            player: Player::First,
            starship,
            delta: UpdateOneDelta::AddOne,
        };

        let result = op.apply(&mut state);
        assert!(matches!(result, Err(OperationError::UnknownStarSystem)));
    }
}
