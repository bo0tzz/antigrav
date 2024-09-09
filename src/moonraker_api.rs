use crate::types::PrinterCommand;
use serde_json::{json, Value};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use websocket::sender::Writer;
use websocket::{ClientBuilder, WebSocketResult};

#[derive(Debug, Clone, Default)]
pub struct PrinterState {
    homed_axes: String,
    absolute_coordinates: bool,
}

struct IdGenerator {
    counter: Arc<Mutex<u64>>,
}

impl IdGenerator {
    fn new(initial: u64) -> Self {
        IdGenerator {
            counter: Arc::new(Mutex::new(initial)),
        }
    }

    fn next_id(&self) -> u64 {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        *counter
    }
}

pub fn connect_to_moonraker(url: &str, printer_rx: Receiver<PrinterCommand>) {
    let client = ClientBuilder::new(url)
        .unwrap()
        .connect_insecure()
        .unwrap();

    let (mut receiver, mut sender) = client.split().unwrap();

    let printer_state = Arc::new(Mutex::new(PrinterState::default()));

    let id_generator = Arc::new(IdGenerator::new(0));

    // Sending thread
    thread::spawn(move || {
        // Subscribe to toolhead and gcode_move objects
        let subscribe_message = json!({
            "jsonrpc": "2.0",
            "method": "printer.objects.subscribe",
            "params": {
                "objects": {
                    "toolhead": null,
                    "gcode_move": null
                }
            },
            "id": id_generator.next_id()
        });
        let _ = send_websocket_message(&mut sender, &subscribe_message);

        for cmd in printer_rx {
            let gcode = cmd.to_string();

            let gcode_message = json!({
                "jsonrpc": "2.0",
                "method": "printer.gcode.script",
                "params": {
                    "script": gcode,
                },
                "id": id_generator.next_id()
            });
            let _ = send_websocket_message(&mut sender, &gcode_message);
        }
    });

    // Receiving thread
    thread::spawn(move || {
        for message in receiver.incoming_messages() {
            match message {
                Ok(websocket::OwnedMessage::Text(text)) => {
                    let json: Value = serde_json::from_str(&text).unwrap();

                    update_printer_state(&printer_state, &json);
                }
                Ok(websocket::OwnedMessage::Close(_)) => {
                    break;
                }
                _ => {}
            }
        }
    });
}

fn update_printer_state(printer_state: &Arc<Mutex<PrinterState>>, json: &Value) {
    let mut state = printer_state.lock().unwrap();

    if let Some(result) = json.get("result") {
        if let Some(status) = result.get("status") {
            update_from_status(&mut state, status);
        }
    } else if json["method"] == "notify_status_update" {
        if let Some(params) = json["params"].as_array() {
            if let Some(status) = params.get(0) {
                update_from_status(&mut state, status);
            }
        }
    }
}

fn update_from_status(state: &mut PrinterState, status: &Value) {
    if let Some(toolhead) = status.get("toolhead") {
        if let Some(homed_axes) = toolhead.get("homed_axes") {
            if let Some(axes) = homed_axes.as_str() {
                state.homed_axes = axes.to_string();
                dbg!(&state);
            }
        }
    }

    if let Some(gcode_move) = status.get("gcode_move") {
        if let Some(absolute_coordinates) = gcode_move.get("absolute_coordinates") {
            if let Some(value) = absolute_coordinates.as_bool() {
                state.absolute_coordinates = value;
                dbg!(&state);
            }
        }
    }


}
fn send_websocket_message(sender: &mut Writer<TcpStream>, message: &Value) -> WebSocketResult<()> {
    println!("send: {}", serde_json::to_string_pretty(&message).unwrap());
    sender.send_message(&websocket::Message::text(message.to_string()))
}
