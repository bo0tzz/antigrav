use crate::types::PrinterCommand;
use spacenav_plus::{Connection, Event, MotionEvent};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub fn start_spacemouse_thread(command_tx: Sender<PrinterCommand>) -> Receiver<MotionEvent> {
    let (motion_tx, motion_rx) = channel();

    thread::spawn(move || {
        spacemouse_event_loop(motion_tx, command_tx);
    });

    motion_rx
}

fn spacemouse_event_loop(motion_tx: Sender<MotionEvent>, command_tx: Sender<PrinterCommand>) {
    let conn = Connection::new().expect("Failed to connect to SpaceMouse");

    loop {
        match conn.wait() {
            Ok(event) => {
                match event {
                    Event::Motion(m) => {
                        motion_tx.send(m).expect("Failed to send SpaceMouse state");
                    },
                    Event::Button(b) => {
                        if b.press {
                            match b.bnum {
                                0 => command_tx.send(PrinterCommand::Home).unwrap(),
                                1 => command_tx.send(PrinterCommand::SetRelativeMotion).unwrap(),
                                _ => eprintln!("unsupported button: {}", b.bnum),
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading SpaceMouse event: {:?}", e),
        }
    }
}
