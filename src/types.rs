use std::fmt::Display;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub enum PrinterCommand {
    Move(MoveParameters),
    Home,
    SetRelativeMotion
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MoveParameters {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub feedrate: i32,
}

impl Display for MoveParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "G0 X{:.3} Y{:.3} Z{:.3} F{}", self.x, self.y, self.z, self.feedrate)
    }
}

pub struct VelocityScaleFactors {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub feedrate: f32,
}