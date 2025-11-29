#![allow(dead_code)]

mod public;
mod engine;

use public::{*, common::Player};


impl common::Player {
    fn opponent(&self) -> Player {
        match &self {
            Player::First => Player::Second,
            Player::Second => Player::First,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
