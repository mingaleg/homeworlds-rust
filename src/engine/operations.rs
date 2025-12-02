mod bank;
mod common;
mod fleet;
mod pending_powers;
mod stars;
mod systems;
mod turn;
mod utils;

use crate::public::*;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

use bank::UpdateBank;
use fleet::UpdateFleet;
use pending_powers::UpdatePendingPowers;
use stars::DestroyStar;
use systems::{DiscoverSystem, ForgetSystem};

#[enum_dispatch]
enum BasicOperation {
    DiscoverSystem,
    ForgetSystem,
    UpdatePendingPowers,
    UpdateFleet,
    UpdateBank,
    DestroyStar,
}

#[derive(Error, Debug)]
enum OperationError {
    #[error("star system with name {name:?} already exists")]
    DuplicatedStarSystemName { name: String },
    #[error("cannot update pending powers")]
    UpdatePendingPowersError(#[from] pending_powers::UpdatePendingPowersError),
    #[error("unknown star system")]
    UnknownStarSystem,
    #[error("cannot update fleet")]
    UpdateFleetError(#[from] fleet::UpdateFleetError),
    #[error("cannot update bank")]
    UpdateBankError(#[from] bank::UpdateBankError),
    #[error("cannot forget system")]
    ForgetSystemError(#[from] systems::ForgetSystemError),
    #[error("cannot destroy star")]
    DestroyStarError(#[from] stars::DestroyStarError),
    #[error("cannot update current turn status")]
    SetCurrentTurnStatusError(#[from] turn::SetCurrentTurnStatusError),
}

#[enum_dispatch(BasicOperation)]
trait Apply {
    fn apply(self, state: &mut current_turn::CurrentTurnState) -> Result<(), OperationError>;
}
