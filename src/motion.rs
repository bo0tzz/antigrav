use crate::constants::{printer_update_interval, PRINTER_TIME_STEP, SCALE_FACTORS};
use crate::types::{MoveParameters};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use spacenav_plus::MotionEvent;
use crate::types::{PrinterCommand, Velocity};

pub fn start_motion_thread(
    spacemouse_rx: Receiver<MotionEvent>,
    printer_tx: Sender<PrinterCommand>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let current_velocity = Arc::new(Mutex::new(Velocity::default()));
        let mut last_handled_velocity = current_velocity.lock().unwrap().clone();

        let _velocity_update_thread = {
            let current_velocity = Arc::clone(&current_velocity);
            thread::spawn(move || {
                loop {
                    if let Ok(state) = spacemouse_rx.recv() {
                        update_velocity(&current_velocity, &state);
                    }
                }
            })
        };

        let mut last_command_time = Instant::now();
        loop {
            let new_velocity = current_velocity.lock().unwrap().clone();
            if last_command_time.elapsed() >= printer_update_interval() && new_velocity != last_handled_velocity {
                last_handled_velocity = new_velocity.clone();
                generate_motion_commands(new_velocity, &printer_tx);
                last_command_time = Instant::now();
            }
            thread::sleep(std::time::Duration::from_millis(1));
        }
    })
}

fn update_velocity(velocity: &Arc<Mutex<Velocity>>, state: &MotionEvent) {
    let mut vel = velocity.lock().unwrap();
    // Update velocity based on SpaceMouse state
    // NOTE!: z and y axes are swapped here to map to the printer's movement system.
    vel.x = state.x as f32 * SCALE_FACTORS.x;
    vel.z = state.y as f32 * SCALE_FACTORS.z;
    vel.y = state.z as f32 * SCALE_FACTORS.y;
}

fn generate_motion_commands(velocity: Velocity, printer_tx: &Sender<PrinterCommand>) {
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

    if let Err(e) = printer_tx.send(command) {
        eprintln!("Failed to send printer command: {:?}", e);
    }
}

fn calculate_feedrate(velocity: &Velocity) -> i32 {
    let speed = (velocity.x.powi(2) + velocity.y.powi(2) + velocity.z.powi(2)).sqrt();
    (speed * 60.0 * SCALE_FACTORS.feedrate) as i32 // Convert to mm/min
}