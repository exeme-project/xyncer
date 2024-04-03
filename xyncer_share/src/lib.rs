use hyper;
use hyper_util;
use log;
use rmp_serde;
use serde::{Deserialize, Serialize};

pub mod payloads;

pub async fn send_payload(
    websocket: &mut fastwebsockets::FragmentCollector<
        hyper_util::rt::TokioIo<hyper::upgrade::Upgraded>,
    >,
    payload: Payload,
) -> Result<(), fastwebsockets::WebSocketError> {
    log::info!("Sent payload: {:?}", payload);

    websocket
        .write_frame(fastwebsockets::Frame::binary(
            fastwebsockets::Payload::Owned(rmp_serde::to_vec(&payload).unwrap()),
        ))
        .await
}

// WebSocket OP codes, in order of most common. Comments show client action and description.
#[derive(Serialize, Deserialize, Debug)]
pub enum OP {
    Dispatch,       // Receive | An event was dispatched
    Heartbeat,      // Send / Receive | Keeps the connection alive
    Identify,       // Send | Starts a new session
    ReIdentify,     // Receive | Re-send an Identify payload with a new passphrase
    InvalidSession, // Receive | The session is invalid
    Hello,          // Receive | Sent immediately after connection
    HeartbeatAck,   // Receive | Acknowledges a heartbeat
}

// WebSocket events
#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    None,
    Ready,
}

// WebSocket payload
#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub op_code: OP,
    pub event_name: Event,
    pub data: payloads::PayloadData,
}
