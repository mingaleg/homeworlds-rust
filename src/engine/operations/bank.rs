use super::common::UpdateOneDelta;
use super::utils;
use super::{Apply, OperationError};
use crate::public::*;
use thiserror::Error;

pub struct UpdateBank {
    pub pyramid: common::Pyramid,
    pub delta: UpdateOneDelta,
}

#[derive(Error, Debug)]
pub enum UpdateBankError {
    #[error("cannot remove a pyramid from the bank as there are none of such type")]
    NoPyramidsInBank,
    #[error("bank count overflow - too many pyramids")]
    BankCountOverflow,
}

impl Apply for UpdateBank {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError> {
        let bank = &mut state.game_board.bank;
        let entry = bank.pyramids.entry(self.pyramid);
        utils::update_hashmap_count(
            entry,
            self.delta,
            UpdateBankError::BankCountOverflow,
            UpdateBankError::NoPyramidsInBank,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateOneDelta;
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
    fn test_add_pyramid_to_empty_bank() {
        let mut state = create_test_state();
        let pyramid = common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        };

        let op = UpdateBank {
            pyramid,
            delta: UpdateOneDelta::AddOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(
            state.game_board.bank.pyramids.get(&pyramid).unwrap().get(),
            1
        );
    }

    #[test]
    fn test_add_pyramid_increment() {
        let mut state = create_test_state();
        let pyramid = common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        };

        state
            .game_board
            .bank
            .pyramids
            .insert(pyramid, NonZero::new(1).unwrap());

        let op = UpdateBank {
            pyramid,
            delta: UpdateOneDelta::AddOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(
            state.game_board.bank.pyramids.get(&pyramid).unwrap().get(),
            2
        );
    }

    #[test]
    fn test_remove_pyramid_decrement() {
        let mut state = create_test_state();
        let pyramid = common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        };

        state
            .game_board
            .bank
            .pyramids
            .insert(pyramid, NonZero::new(2).unwrap());

        let op = UpdateBank {
            pyramid,
            delta: UpdateOneDelta::RemoveOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(
            state.game_board.bank.pyramids.get(&pyramid).unwrap().get(),
            1
        );
    }

    #[test]
    fn test_remove_pyramid_removes_entry() {
        let mut state = create_test_state();
        let pyramid = common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        };

        state
            .game_board
            .bank
            .pyramids
            .insert(pyramid, NonZero::new(1).unwrap());

        let op = UpdateBank {
            pyramid,
            delta: UpdateOneDelta::RemoveOne,
        };

        let result = op.apply(&mut state);
        assert!(result.is_ok());
        assert!(!state.game_board.bank.pyramids.contains_key(&pyramid));
    }

    #[test]
    fn test_remove_nonexistent_pyramid() {
        let mut state = create_test_state();
        let pyramid = common::Pyramid {
            size: common::Size::Small,
            color: common::Color::Red,
        };

        let op = UpdateBank {
            pyramid,
            delta: UpdateOneDelta::RemoveOne,
        };

        let result = op.apply(&mut state);
        assert!(matches!(
            result,
            Err(OperationError::UpdateBankError(
                UpdateBankError::NoPyramidsInBank
            ))
        ));
    }
}
