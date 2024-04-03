use anyhow::Error;

pub struct Session {
    pub authenticated: bool,
    pub connected: bool,

    pub error: Error,
    pub password: String,

    pub payload_sender: tokio::sync::mpsc::Sender<xyncer_share::Payload>,
    pub payload_receiver: tokio::sync::mpsc::Receiver<xyncer_share::Payload>,

    pub server_address: String,
}
