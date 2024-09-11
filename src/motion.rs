use crate::constants::{printer_update_interval, PRINTER_TIME_STEP, SCALE_FACTORS};
use crate::types::{MoveParameters, PrinterCommand, Velocity};
use spacenav_plus::MotionEvent;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;
use std::sync::Arc;
use std::time::Instant;

pub async fn start_motion_thread(
    mut spacemouse_rx: Receiver<MotionEvent>,
    printer_tx: Sender<PrinterCommand>,
) {
    let current_velocity = Arc::new(Mutex::new(Velocity::default()));

    let vel = Arc::clone(&current_velocity);
    task::spawn(async move {
        while let Some(state) = spacemouse_rx.recv().await {
            let mut v = vel.lock().await;
            // Update velocity based on SpaceMouse state
            // NOTE!: z and y axes are swapped here to map to the printer's movement system.
            v.x = state.x as f32 * SCALE_FACTORS.x;
            v.z = state.y as f32 * SCALE_FACTORS.z;
            v.y = state.z as f32 * SCALE_FACTORS.y;
        }
    });

    let mut last_command_time = Instant::now();
    let mut last_handled_velocity = Velocity::default();
    task::spawn(async move {
        loop {
            let new_velocity = current_velocity.lock().await.clone();
            if last_command_time.elapsed() >= printer_update_interval() && new_velocity != last_handled_velocity {
                last_handled_velocity = new_velocity.clone();
                generate_motion_commands(new_velocity, &printer_tx).await;
                last_command_time = Instant::now();
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
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
        eprintln!("Failed to send printer command: {:?}", e);
    }
}

fn calculate_feedrate(velocity: &Velocity) -> i32 {
    let speed = (velocity.x.powi(2) + velocity.y.powi(2) + velocity.z.powi(2)).sqrt();
    (speed * 60.0 * SCALE_FACTORS.feedrate) as i32 // Convert to mm/min
}