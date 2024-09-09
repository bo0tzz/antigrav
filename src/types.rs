use std::fmt::Display;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PrinterCommand {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub feedrate: i32,
}

impl Display for PrinterCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "G0 X{:.3} Y{:.3} Z{:.3} F{}", self.x, self.y, self.z, self.feedrate)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SpaceMouseState {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub button0: bool,
    pub button1: bool,
}

impl Default for SpaceMouseState {
    fn default() -> Self {
        SpaceMouseState {
            x: 0,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            button0: false,
            button1: false,
        }
    }
}

pub struct VelocityScaleFactors {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub feedrate: f32,
}