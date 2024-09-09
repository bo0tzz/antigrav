// constants.rs

use crate::types::VelocityScaleFactors;

pub const SCALE_FACTORS: VelocityScaleFactors = VelocityScaleFactors {
    x: 0.5,
    y: 0.5,
    z: 0.3,
    feedrate: 1.0
};

pub const PRINTER_UPDATE_RATE_HZ: f32 = 5.0;
pub const PRINTER_TIME_STEP: f32 = 1.0 / PRINTER_UPDATE_RATE_HZ;
pub fn printer_update_interval() -> std::time::Duration {
    std::time::Duration::from_secs_f32(1.0 / PRINTER_UPDATE_RATE_HZ)
}
