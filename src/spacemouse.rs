use crate::types::PrinterCommand;
use spacenav_plus::{Connection, Event, MotionEvent};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;

pub async fn start_spacemouse_thread(command_tx: Sender<PrinterCommand>) -> Receiver<MotionEvent> {
    let (motion_tx, motion_rx) = channel(32);

    task::spawn(async move {
        let conn = Connection::new().expect("Failed to connect to SpaceMouse");

        loop {
            match conn.wait() {
                Ok(event) => {
                    log::debug!("Received SpaceMouse event: {:?}", event);
                    match event {
                        Event::Motion(m) => {
                            motion_tx.send(m).await.expect("Failed to send SpaceMouse state");
                            log::debug!("Sent SpaceMouse state");
                        },
                        Event::Button(b) => {
                            if b.press {
                                match b.bnum {
                                    0 => command_tx.send(PrinterCommand::Home).await.unwrap(),
                                    1 => command_tx.send(PrinterCommand::SetRelativeMotion).await.unwrap(),
                                    _ => log::warn!("unsupported button: {}", b.bnum),
                                }
                            }
                        }
                    }
                }
                Err(e) => log::error!("Error reading SpaceMouse event: {:?}", e),
            }
        }
    });

    motion_rx
}