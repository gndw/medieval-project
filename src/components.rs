#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug)]
pub struct Name(pub String);
