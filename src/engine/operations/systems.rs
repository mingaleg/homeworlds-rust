use super::{Apply, OperationError};
use crate::public::*;
use thiserror::Error;

pub struct DiscoverSystem {
    pub name: String,
    pub center_star: board::Star,
}

impl Apply for DiscoverSystem {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let game_board = &mut state.game_board;
        if game_board
            .discovered_systems
            .iter()
            .any(|it| it.name == self.name)
        {
            return Err(OperationError::DuplicatedStarSystemName { name: self.name });
        }
        game_board.discovered_systems.push(board::StarSystem {
            name: self.name,
            center: board::StarSystemCenter::SingleStar(self.center_star),
            fleet_first: board::Fleet::default(),
            fleet_second: board::Fleet::default(),
            is_homeworld_for: None,
        });
        Ok(())
    }
}

pub struct ForgetSystem {
    pub star_system_name: String,
}

#[derive(Error, Debug)]
pub enum ForgetSystemError {
    #[error("cannot forget a homeworld")]
    CannotForgetHomeworld,
    #[error("cannot forget a system with non-empty fleets")]
    FleetsNotEmpty,
}

impl Apply for ForgetSystem {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let system_index = state
            .game_board
            .discovered_systems
            .iter()
            .position(|it| it.name == self.star_system_name)
            .ok_or(OperationError::UnknownStarSystem)?;

        let system = &state.game_board.discovered_systems[system_index];

        if system.is_homeworld_for.is_some() {
            return Err(ForgetSystemError::CannotForgetHomeworld.into());
        }

        if !system.fleet_first.starships.is_empty() || !system.fleet_second.starships.is_empty() {
            return Err(ForgetSystemError::FleetsNotEmpty.into());
        }

        state.game_board.discovered_systems.remove(system_index);
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
    fn test_discover_system_success() {
        let mut state = create_test_state();
        let op = DiscoverSystem {
            name: "Alpha".to_string(),
            center_star: board::Star(common::Pyramid {
                size: common::Size::Small,
                color: common::Color::Red,
            }),
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.game_board.discovered_systems.len(), 1);
        assert_eq!(state.game_board.discovered_systems[0].name, "Alpha");
    }

    #[test]
    fn test_discover_system_duplicate_name() {
        let mut state = create_test_state();
        state.game_board.discovered_systems.push(board::StarSystem {
            name: "Alpha".to_string(),
            center: board::StarSystemCenter::Empty,
            fleet_first: board::Fleet::default(),
            fleet_second: board::Fleet::default(),
            is_homeworld_for: None,
        });

        let op = DiscoverSystem {
            name: "Alpha".to_string(),
            center_star: board::Star(common::Pyramid {
                size: common::Size::Small,
                color: common::Color::Red,
            }),
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::DuplicatedStarSystemName { .. })
        ));
    }

    #[test]
    fn test_forget_system_success() {
        let mut state = create_test_state();
        state.game_board.discovered_systems.push(board::StarSystem {
            name: "Alpha".to_string(),
            center: board::StarSystemCenter::Empty,
            fleet_first: board::Fleet::default(),
            fleet_second: board::Fleet::default(),
            is_homeworld_for: None,
        });

        let op = ForgetSystem {
            star_system_name: "Alpha".to_string(),
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.game_board.discovered_systems.len(), 0);
    }

    #[test]
    fn test_forget_system_unknown() {
        let mut state = create_test_state();
        let op = ForgetSystem {
            star_system_name: "Unknown".to_string(),
        };

        let result = op.apply(&mut state);
        assert!(matches!(result, Err(OperationError::UnknownStarSystem)));
    }

    #[test]
    fn test_forget_system_homeworld() {
        let mut state = create_test_state();
        state.game_board.discovered_systems.push(board::StarSystem {
            name: "Homeworld".to_string(),
            center: board::StarSystemCenter::Empty,
            fleet_first: board::Fleet::default(),
            fleet_second: board::Fleet::default(),
            is_homeworld_for: Some(common::Player::First),
        });

        let op = ForgetSystem {
            star_system_name: "Homeworld".to_string(),
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::ForgetSystemError(
                ForgetSystemError::CannotForgetHomeworld
            ))
        ));
    }

    #[test]
    fn test_forget_system_non_empty_fleet() {
        let mut state = create_test_state();
        let mut fleet = board::Fleet::default();
        fleet.starships.insert(
            board::Starship(common::Pyramid {
                size: common::Size::Small,
                color: common::Color::Red,
            }),
            NonZero::new(1).unwrap(),
        );

        state.game_board.discovered_systems.push(board::StarSystem {
            name: "Alpha".to_string(),
            center: board::StarSystemCenter::Empty,
            fleet_first: fleet,
            fleet_second: board::Fleet::default(),
            is_homeworld_for: None,
        });

        let op = ForgetSystem {
            star_system_name: "Alpha".to_string(),
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::ForgetSystemError(
                ForgetSystemError::FleetsNotEmpty
            ))
        ));
    }
}
