use crate::types::PrinterCommand;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use futures::{StreamExt};
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct PrinterState {
    homed_axes: String,
    absolute_coordinates: bool,
}

struct IdGenerator {
    counter: u64,
}

impl IdGenerator {
    fn new(initial: u64) -> Self {
        IdGenerator { counter: initial }
    }

    fn next_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}

struct RpcTracker {
    start_times: Mutex<HashMap<u64, Instant>>,
}

impl RpcTracker {
    fn new() -> Self {
        RpcTracker {
            start_times: Mutex::new(HashMap::new()),
        }
    }

    async fn start_tracking(&self, id: u64) {
        let mut start_times = self.start_times.lock().await;
        start_times.insert(id, Instant::now());
    }

    async fn stop_tracking(&self, id: u64) -> Option<Duration> {
        let mut start_times = self.start_times.lock().await;
        start_times.remove(&id).map(|start| start.elapsed())
    }
}

pub async fn connect_to_moonraker(url: &str, mut printer_rx: Receiver<PrinterCommand>) {
    let (ws_stream, _) = connect_async(url).await.expect("Can't connect");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let mut id_generator = IdGenerator::new(0);
    let mut printer_state = PrinterState::default();
    let rpc_tracker = RpcTracker::new();

    // Subscribe to toolhead and gcode_move objects
    let subscribe_params = json!({
            "objects": {
                "toolhead": null,
                "gcode_move": null
            }
    });
    send_rpc(&mut ws_sender, &mut id_generator, &rpc_tracker, "printer.objects.subscribe", subscribe_params).await.unwrap();

    loop {
        tokio::select! {
            Some(cmd) = printer_rx.recv() => {
                match cmd {
                    PrinterCommand::Move(m) => {
                        println!("Received move command: {:?}", m);
                        if printer_state.absolute_coordinates || printer_state.homed_axes != "xyz" {
                            eprintln!("Refusing move in bad printer state: {:?}", printer_state)
                        } else {
                            send_gcode_command(&mut ws_sender, &mut id_generator, &rpc_tracker, &m.to_string()).await.expect("Failed to send move command");
                        }
                    }
                    PrinterCommand::Home => {send_gcode_command(&mut ws_sender, &mut id_generator, &rpc_tracker, "G28").await.expect("Failed to send home command");}
                    PrinterCommand::SetRelativeMotion =>{ send_gcode_command(&mut ws_sender, &mut id_generator, &rpc_tracker, "G91").await.expect("Failed to send relative motion command");}
                }
            }
            Some(Ok(msg)) = ws_receiver.next() => {
                match msg {
                    Message::Text(text) => {
                        let json: Value = serde_json::from_str(&text).unwrap();
                        log_recv(&json);
                        if let Some(id) = json.get("id").and_then(|id| id.as_u64()) {
                            if let Some(duration) = rpc_tracker.stop_tracking(id).await {
                                println!("RPC call with id {} took {:?}", id, duration);
                            }
                        }
                        update_printer_state(&mut printer_state, &json).await;
                    }
                    Message::Close(_) => break,
                    Message::Ping(_) => println!("Ping!"),
                    other => eprintln!("Unexpected websocket message: {:?}", other),
                }
            }
            else => break,
        }
    }
}

fn log_recv(json: &Value) {
    if let Some(method) = json.get("method") {
        match method.as_str().unwrap() {
            "notify_status_update" => {}
            "notify_proc_stat_update" => {}
            "notify_service_state_changed" => {}
            _ => println!("recv: {}", serde_json::to_string_pretty(&json).unwrap())
        }
    } else {
        println!("recv: {}", serde_json::to_string_pretty(&json).unwrap())
    }
}

async fn update_printer_state(printer_state: &mut PrinterState, json: &Value) {
    if let Some(result) = json.get("result") {
        if let Some(status) = result.get("status") {
            update_from_status(printer_state, status);
        }
    } else if json["method"] == "notify_status_update" {
        if let Some(params) = json["params"].as_array() {
            if let Some(status) = params.get(0) {
                update_from_status(printer_state, status);
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

async fn send_rpc(
    sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    id_generator: &mut IdGenerator,
    rpc_tracker: &RpcTracker,
    method: &str,
    params: Value,
) -> Result<u64, Error> {
    let id = id_generator.next_id();
    let rpc_message = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id
    });
    rpc_tracker.start_tracking(id).await;
    sender.send(Message::Text(rpc_message.to_string())).await?;
    Ok(id)
}

async fn send_gcode_command(
    sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    id_generator: &mut IdGenerator,
    rpc_tracker: &RpcTracker,
    gcode: &str,
) -> Result<u64, Error> {
    println!("Sending GCode: {}", gcode);
    send_rpc(sender, id_generator, rpc_tracker, "printer.gcode.script", json!({
        "script": gcode,
    })).await
}