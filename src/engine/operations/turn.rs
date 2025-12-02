use super::{Apply, OperationError};
use crate::public::current_turn::CurrentTurnStatus;
use crate::public::*;
use thiserror::Error;

pub struct SetCurrentTurnStatus {
    pub new_status: CurrentTurnStatus,
}

#[derive(Error, Debug)]
pub enum SetCurrentTurnStatusError {
    #[error("can only change current turn status from MakingActions")]
    CanOnlyChangeFromMakingActions,
    #[error("tried change the current turn status to the same value")]
    NoChange,
}

impl Apply for SetCurrentTurnStatus {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        if state.current_turn_status == self.new_status {
            return Err(SetCurrentTurnStatusError::NoChange.into());
        }
        if state.current_turn_status != current_turn::CurrentTurnStatus::MakingActions {
            return Err(SetCurrentTurnStatusError::CanOnlyChangeFromMakingActions.into());
        }
        state.current_turn_status = self.new_status;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::public::{board, common, current_turn};

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
    fn test_set_to_passing() {
        let mut state = create_test_state();
        let op = SetCurrentTurnStatus {
            new_status: CurrentTurnStatus::Passing,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(state.current_turn_status == CurrentTurnStatus::Passing);
    }

    #[test]
    fn test_set_to_resigning() {
        let mut state = create_test_state();
        let op = SetCurrentTurnStatus {
            new_status: CurrentTurnStatus::Resigning,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(state.current_turn_status == CurrentTurnStatus::Resigning);
    }

    #[test]
    fn test_set_to_same() {
        use strum::IntoEnumIterator;
        for status in CurrentTurnStatus::iter() {
            let mut state = create_test_state();
            state.current_turn_status = status.clone();
            let op = SetCurrentTurnStatus {
                new_status: status.clone(),
            };

            let result = op.apply(&mut state);
            assert!(matches!(
                result.err(),
                Some(OperationError::SetCurrentTurnStatusError(
                    SetCurrentTurnStatusError::NoChange
                ))
            ));
            assert!(state.current_turn_status == status);
        }
    }

    #[test]
    fn test_set_not_from_making_actions() {
        use strum::IntoEnumIterator;
        for status in CurrentTurnStatus::iter() {
            if status == CurrentTurnStatus::MakingActions {
                continue;
            }
            let mut state = create_test_state();
            state.current_turn_status = status.clone();
            let op = SetCurrentTurnStatus {
                new_status: CurrentTurnStatus::MakingActions,
            };

            let result = op.apply(&mut state);
            assert!(matches!(
                result.err(),
                Some(OperationError::SetCurrentTurnStatusError(
                    SetCurrentTurnStatusError::CanOnlyChangeFromMakingActions
                ))
            ));
            assert!(state.current_turn_status == status);
        }
    }
}
