use std::mem::replace;

use super::{Apply, OperationError};
use crate::public::*;
use thiserror::Error;

pub enum DestroyStarSelector {
    Binary(board::BinaryStarId),
    Single,
}

pub struct DestroyStar {
    pub star_system_name: String,
    pub star: DestroyStarSelector,
}

#[derive(Error, Debug)]
pub enum DestroyStarError {
    #[error("star system center is empty")]
    CenterAlreadyEmpty,
    #[error("cannot destroy binary star from a single star system")]
    NotABinarySystem,
    #[error("cannot destroy single star from a binary system")]
    NotASingleStarSystem,
}

impl Apply for DestroyStar {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let Some(system) = state
            .game_board
            .discovered_systems
            .iter_mut()
            .find(|it| it.name == self.star_system_name)
        else {
            return Err(OperationError::UnknownStarSystem);
        };

        let old_center = replace(&mut system.center, board::StarSystemCenter::Empty);

        system.center = match self.star {
            DestroyStarSelector::Binary(star_id) => {
                match old_center {
                    board::StarSystemCenter::BinaryStar { alpha, beta } => {
                        // Keep the star that wasn't destroyed
                        let remaining_star = match star_id {
                            board::BinaryStarId::Alpha => beta,
                            board::BinaryStarId::Beta => alpha,
                        };
                        board::StarSystemCenter::SingleStar(remaining_star)
                    }
                    board::StarSystemCenter::SingleStar(_) => {
                        return Err(DestroyStarError::NotABinarySystem.into());
                    }
                    board::StarSystemCenter::Empty => {
                        return Err(DestroyStarError::CenterAlreadyEmpty.into());
                    }
                }
            }
            DestroyStarSelector::Single => match old_center {
                board::StarSystemCenter::SingleStar(_) => board::StarSystemCenter::Empty,
                board::StarSystemCenter::BinaryStar { .. } => {
                    return Err(DestroyStarError::NotASingleStarSystem.into());
                }
                board::StarSystemCenter::Empty => {
                    return Err(DestroyStarError::CenterAlreadyEmpty.into());
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
    fn test_destroy_binary_star_alpha() {
        let mut state = create_test_state();
        let alpha = board::Star(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });
        let beta = board::Star(common::Pyramid {
            size: common::Size::Medium,
            color: common::Color::Blue,
        });

        state.game_board.discovered_systems[0].center =
            board::StarSystemCenter::BinaryStar { alpha, beta };

        let op = DestroyStar {
            star_system_name: "Alpha".to_string(),
            star: DestroyStarSelector::Binary(board::BinaryStarId::Alpha),
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(matches!(
            state.game_board.discovered_systems[0].center,
            board::StarSystemCenter::SingleStar(_)
        ));
    }

    #[test]
    fn test_destroy_binary_star_beta() {
        let mut state = create_test_state();
        let alpha = board::Star(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });
        let beta = board::Star(common::Pyramid {
            size: common::Size::Medium,
            color: common::Color::Blue,
        });

        state.game_board.discovered_systems[0].center =
            board::StarSystemCenter::BinaryStar { alpha, beta };

        let op = DestroyStar {
            star_system_name: "Alpha".to_string(),
            star: DestroyStarSelector::Binary(board::BinaryStarId::Beta),
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(matches!(
            state.game_board.discovered_systems[0].center,
            board::StarSystemCenter::SingleStar(_)
        ));
    }

    #[test]
    fn test_destroy_single_star() {
        let mut state = create_test_state();
        state.game_board.discovered_systems[0].center =
            board::StarSystemCenter::SingleStar(board::Star(common::Pyramid {
                size: common::Size::Small,
                color: common::Color::Red,
            }));

        let op = DestroyStar {
            star_system_name: "Alpha".to_string(),
            star: DestroyStarSelector::Single,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(matches!(
            state.game_board.discovered_systems[0].center,
            board::StarSystemCenter::Empty
        ));
    }

    #[test]
    fn test_destroy_binary_when_single() {
        let mut state = create_test_state();
        state.game_board.discovered_systems[0].center =
            board::StarSystemCenter::SingleStar(board::Star(common::Pyramid {
                size: common::Size::Small,
                color: common::Color::Red,
            }));

        let op = DestroyStar {
            star_system_name: "Alpha".to_string(),
            star: DestroyStarSelector::Binary(board::BinaryStarId::Alpha),
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::DestroyStarError(
                DestroyStarError::NotABinarySystem
            ))
        ));
    }

    #[test]
    fn test_destroy_single_when_binary() {
        let mut state = create_test_state();
        let alpha = board::Star(common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        });
        let beta = board::Star(common::Pyramid {
            size: common::Size::Medium,
            color: common::Color::Blue,
        });

        state.game_board.discovered_systems[0].center =
            board::StarSystemCenter::BinaryStar { alpha, beta };

        let op = DestroyStar {
            star_system_name: "Alpha".to_string(),
            star: DestroyStarSelector::Single,
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::DestroyStarError(
                DestroyStarError::NotASingleStarSystem
            ))
        ));
    }

    #[test]
    fn test_destroy_when_already_empty() {
        let mut state = create_test_state();

        let op = DestroyStar {
            star_system_name: "Alpha".to_string(),
            star: DestroyStarSelector::Single,
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::DestroyStarError(
                DestroyStarError::CenterAlreadyEmpty
            ))
        ));
    }

    #[test]
    fn test_unknown_star_system() {
        let mut state = create_test_state();

        let op = DestroyStar {
            star_system_name: "Unknown".to_string(),
            star: DestroyStarSelector::Single,
        };

        let result = op.apply(&mut state);
        assert!(matches!(result, Err(OperationError::UnknownStarSystem)));
    }
}
