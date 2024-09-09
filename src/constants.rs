// constants.rs

use crate::types::VelocityScaleFactors;

pub const SCALE_FACTORS: VelocityScaleFactors = VelocityScaleFactors {
    x: 1.0,
    y: 1.0,
    z: 0.5,
    feedrate: 0.4
};

pub const PRINTER_UPDATE_RATE_HZ: f32 = 30.0;
pub const PRINTER_TIME_STEP: f32 = 1.0 / PRINTER_UPDATE_RATE_HZ;
pub fn printer_update_interval() -> std::time::Duration {
    std::time::Duration::from_secs_f32(1.0 / PRINTER_UPDATE_RATE_HZ)
}
