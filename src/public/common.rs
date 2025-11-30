#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub enum Color {
    Green,
    Yellow,
    Red,
    Blue,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
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

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct Pyramid {
    pub color: Color,
    pub size: Size,
}

pub enum Player {
    First,
    Second,
}
