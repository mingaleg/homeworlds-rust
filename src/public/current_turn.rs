use super::board::*;
use super::common::*;
use std::num::NonZero;
use strum_macros::EnumIter;

pub enum PendingPowers {
    Nil,
    Pending {
        power: Power,
        count: NonZero<u8>,
        original_count: NonZero<u8>,
    },
    Exhausted {
        power: Power,
        original_count: NonZero<u8>,
    },
}

#[derive(Eq, PartialEq, EnumIter, Clone)]
pub enum CurrentTurnStatus {
    MakingActions,
    Passing,
    Resigning,
}

pub struct CurrentTurnState {
    pub player: Player,
    pub game_board: GameBoard,
    pub pending_powers: PendingPowers,
    pub current_turn_status: CurrentTurnStatus,
}
