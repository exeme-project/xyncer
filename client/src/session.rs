use flume;

#[derive(Clone)]
pub struct Session {
    pub authenticated: bool,
    pub connected: bool,

    pub error: Option<String>,
    pub password: String,

    pub payload_sender: flume::Sender<xyncer_share::Payload>,
    pub payload_receiver: flume::Receiver<xyncer_share::Payload>,

    pub server_address: String,
}
