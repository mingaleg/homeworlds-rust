use std::collections::HashMap;
use std::num::NonZero;
use super::common::*;

pub struct Star(Pyramid);

pub enum BinaryStarId {
    Alpha,
    Beta,
}

pub enum StarSystemCenter {
    Empty,
    SingleStar(Star),
    BinaryStar{alpha: Star, beta: Star},
}

#[derive(Eq, Hash, PartialEq)]
pub struct Starship(pub Pyramid);

#[derive(Default)]
pub struct Fleet {
    pub starships: HashMap<Starship, NonZero<u8>>,
}

pub struct StarSystem {
    pub name: String,
    pub center: StarSystemCenter,
    pub fleet_first: Fleet,
    pub fleet_second: Fleet,
}

impl StarSystem {
    pub fn fleet(&self, player: Player) -> &Fleet {
        match player {
            Player::First => &self.fleet_first,
            Player::Second => &self.fleet_second,
        }
    }

    pub fn fleet_mut(&mut self, player: Player) -> &mut Fleet {
        match player {
            Player::First => &mut self.fleet_first,
            Player::Second => &mut self.fleet_second,
        }
    }
}

pub struct Bank {
    pub pyramids: HashMap<Pyramid, NonZero<u8>>,
}

pub struct GameBoard {
    pub bank: Bank,
    pub homeworld_first: StarSystem,
    pub homeworld_second: StarSystem,
    pub discovered_systems: Vec<StarSystem>,
}
