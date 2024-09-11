use crate::constants::{printer_update_interval, PRINTER_TIME_STEP, SCALE_FACTORS};
use log::debug;
use crate::types::{MoveParameters, PrinterCommand, Velocity};
use spacenav_plus::MotionEvent;
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;

pub async fn start_motion_thread(
    mut spacemouse_rx: Receiver<MotionEvent>,
    printer_tx: Sender<PrinterCommand>,
) {
    let mut current_velocity = Velocity::default();
    let mut last_command_time = Instant::now();
    let mut last_handled_velocity = Velocity::default();

    task::spawn(async move {
        loop {
            tokio::select! {
            Some(state) = spacemouse_rx.recv() => {
                // Update velocity based on SpaceMouse state
                // NOTE!: z and y axes are swapped here to map to the printer's movement system.
                current_velocity.x = state.x as f32 * SCALE_FACTORS.x;
                current_velocity.z = state.y as f32 * SCALE_FACTORS.z;
                current_velocity.y = state.z as f32 * SCALE_FACTORS.y;
                debug!("Updated velocity: {:?}", current_velocity);
            }
            else => {
                if last_command_time.elapsed() >= printer_update_interval() && current_velocity != last_handled_velocity {
                    last_handled_velocity = current_velocity.clone();
                    generate_motion_commands(current_velocity.clone(), &printer_tx).await;
                    debug!("Generated motion commands for velocity: {:?}", current_velocity);
                last_command_time = Instant::now();
            }
            }
        }
        }
    });
}


async fn generate_motion_commands(velocity: Velocity, printer_tx: &Sender<PrinterCommand>) {
    let feedrate = calculate_feedrate(&velocity);
    if feedrate == 0 {
        return;
    }
    let command = PrinterCommand::Move(MoveParameters {
        x: velocity.x * PRINTER_TIME_STEP,
        y: velocity.y * PRINTER_TIME_STEP,
        z: velocity.z * PRINTER_TIME_STEP,
        feedrate,
    });

    if let Err(e) = printer_tx.send(command).await {
        log::error!("Failed to send printer command: {:?}", e);
    }
}

fn calculate_feedrate(velocity: &Velocity) -> i32 {
    let speed = (velocity.x.powi(2) + velocity.y.powi(2) + velocity.z.powi(2)).sqrt();
    (speed * 60.0 * SCALE_FACTORS.feedrate) as i32 // Convert to mm/min
}