use super::board::*;
use super::common::*;

pub enum MoveTargetStarSystem {
    Known { star_system: StarSystem },
    Discovered,
}

pub enum ActionInStarSystem {
    Build {
        color: Color,
    },
    Move {
        starship: Starship,
        target: MoveTargetStarSystem,
    },
    Capture {
        starship: Starship,
    },
    Trade {
        starship: Starship,
        new_color: Color,
    },
    DeclareCatastrophe {
        color: Color,
    },
    Sacrifice {
        starship: Starship,
    },
}

pub enum Action {
    Play {
        star_system: StarSystem,
        action: Box<ActionInStarSystem>,
    },
    Pass,
    Resign,
}
