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

    let (mut receiver, s) = client.split().unwrap();
    let sender = Arc::new(Mutex::new(s));
    let pingpong_sender = Arc::clone(&sender);

    let printer_state = Arc::new(Mutex::new(PrinterState::default()));
    let sender_state = Arc::clone(&printer_state);
    let receiver_state = Arc::clone(&printer_state);

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
        let mut s = sender.lock().unwrap();
        let _ = send_websocket_message(&mut s, &subscribe_message);

        for cmd in printer_rx {
            send_gcode_command(&mut s, Arc::clone(&id_generator), Arc::clone(&sender_state), cmd);
        }
    });

    // Receiving thread
    thread::spawn(move || {
        for message in receiver.incoming_messages() {
            match message {
                Ok(websocket::OwnedMessage::Text(text)) => {
                    let json: Value = serde_json::from_str(&text).unwrap();
                    log_recv(&json);
                    update_printer_state(&receiver_state, &json);
                }
                Ok(websocket::OwnedMessage::Close(_)) => {
                    break;
                }
                Ok(websocket::OwnedMessage::Ping(d)) => {
                    println!("Ping!");
                    pingpong_sender.lock().unwrap().send_message(&websocket::Message::pong(d)).unwrap();
                }
                Ok(other) => {
                    eprintln!("Unexpected websocket message: {:?}", other);
                }
                Err(e) => {
                    eprintln!("websocket error: {}", e);
                }
            }
        }
    });
}

fn log_recv(json: &Value) {
    if let Some(method) = json.get("method") {
        match method.as_str().unwrap() {
            "notify_status_update" => {},
            "notify_proc_stat_update" => {},
            _ => {} //println!("recv: {}", serde_json::to_string_pretty(&json).unwrap())
        }
    }
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

fn send_gcode_command(mut sender: &mut Writer<TcpStream>, id_generator: Arc<IdGenerator>, printer_state: Arc<Mutex<PrinterState>>, command: PrinterCommand) {
    let mut send_msg = |gcode| {
        let gcode_message = json!({
                "jsonrpc": "2.0",
                "method": "printer.gcode.script",
                "params": {
                    "script": gcode,
                },
                "id": id_generator.next_id()
            });
        let _ = send_websocket_message(&mut sender, &gcode_message);
    };
    match command {
        PrinterCommand::Move(m) => {
            let state = printer_state.lock().unwrap();
            if state.absolute_coordinates || state.homed_axes != "xyz" {
                eprintln!("Refusing move in bad printer state: {:?}", state)
            } else {
                send_msg(m.to_string())
            }
        }
        PrinterCommand::Home => { send_msg("G28".to_string()) }
        PrinterCommand::SetRelativeMotion => { send_msg("G91".to_string()) }
    }
}
fn send_websocket_message(sender: &mut Writer<TcpStream>, message: &Value) -> WebSocketResult<()> {
    println!("send: {}", serde_json::to_string_pretty(&message).unwrap());
    sender.send_message(&websocket::Message::text(message.to_string()))
}
