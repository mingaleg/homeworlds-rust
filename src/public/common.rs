#[derive(Eq, Hash, PartialEq)]
pub enum Color {
    Green,
    Yellow,
    Red,
    Blue,
}

#[derive(Eq, Hash, PartialEq)]
pub enum Size {
    Small,
    Medium,
    Large,
}

pub enum Power {
    Build,
    Move,
    Captute,
    Trade,
}

#[derive(Eq, Hash, PartialEq)]
pub struct Pyramid {
    color: Color,
    size: Size,
}

pub enum Player {
    First,
    Second,
}
