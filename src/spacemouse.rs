use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use spacenav_plus::{ButtonEvent, Connection, Event, MotionEvent};
use crate::types::SpaceMouseState;

pub fn start_spacemouse_thread() -> Receiver<SpaceMouseState> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        spacemouse_event_loop(tx);
    });

    rx
}

fn spacemouse_event_loop(tx: Sender<SpaceMouseState>) {
    let conn = Connection::new().expect("Failed to connect to SpaceMouse");
    let mut state = SpaceMouseState::default();

    loop {
        match conn.wait() {
            Ok(event) => {
                match event {
                    Event::Motion(m) => handle_motion(&mut state, m),
                    Event::Button(b) => handle_button(&mut state, b),
                }
                tx.send(state.clone()).expect("Failed to send SpaceMouse state");
            }
            Err(e) => eprintln!("Error reading SpaceMouse event: {:?}", e),
        }
    }
}

fn handle_motion(state: &mut SpaceMouseState, m: MotionEvent) {
    state.x = m.x;
    state.y = m.y;
    state.z = m.z;
    state.rx = m.rx;
    state.ry = m.ry;
    state.rz = m.rz;
}

fn handle_button(state: &mut SpaceMouseState, b: ButtonEvent) {
    match b.bnum {
        0 => state.button0 = b.press,
        1 => state.button1 = b.press,
        _ => eprintln!("unsupported button: {}", b.bnum),
    }
}
