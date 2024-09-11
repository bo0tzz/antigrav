// src/main.rs
mod spacemouse;
mod motion;
mod constants;
mod types;
mod moonraker_api;

use motion::start_motion_thread;
use spacemouse::start_spacemouse_thread;
use tokio::sync::mpsc;
use crate::moonraker_api::connect_to_moonraker;

#[tokio::main]
async fn main() {
    // Create a channel for printer commands
    let (printer_tx, printer_rx) = mpsc::channel(32);

    // Start the SpaceMouse thread and get the receiver
    let spacemouse_rx = start_spacemouse_thread(printer_tx.clone()).await;

    // Start the motion command generation thread
    start_motion_thread(spacemouse_rx, printer_tx.clone()).await;

    // Create and start the MoonrakerApi communication thread
    connect_to_moonraker("ws://192.168.178.235:7125/websocket", printer_rx).await;

    // Implement proper shutdown mechanism here
    // For example, send a shutdown signal to threads and wait for them to finish

}