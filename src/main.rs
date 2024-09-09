mod spacemouse;
mod motion;
mod constants;
mod types;

use motion::start_motion_thread;
use spacemouse::start_spacemouse_thread;
use std::sync::mpsc;
use std::thread;

fn main() {
    // Start the SpaceMouse thread and get the receiver
    let spacemouse_rx = start_spacemouse_thread();

    // Create a channel for printer commands
    let (printer_tx, printer_rx) = mpsc::channel();

    // Start the motion command generation thread
    let motion_thread = start_motion_thread(spacemouse_rx, printer_tx.clone());

    // Printer API communication thread
    let printer_thread = thread::spawn(move || {
        loop {
            if let Ok(command) = printer_rx.recv() {
                println!("{}", command.to_string());
            }
        }
    });

    // Implement proper shutdown mechanism here
    // For example, send a shutdown signal to threads and wait for them to finish

    // Wait for threads
    motion_thread.join().unwrap();
    printer_thread.join().unwrap();
}
