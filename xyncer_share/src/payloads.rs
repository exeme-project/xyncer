use serde::{Deserialize, Serialize};

// Dispatch data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DispatchData {
    pub data: String,
}

// Identify data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentifyData {
    pub passphrase: String,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
enum ErrorCode {
    UnknownError,
    UnknownOP,
    DecodeError,
    AuthenticationFailed,
}

impl ErrorCode {
    fn populate(&self) -> InvalidSessionData {
        match self {
            ErrorCode::UnknownError => InvalidSessionData {
                code: *self,
                description: "Unknown error".to_string(),
                explanation: "We're not sure what went wrong. Try reconnecting?".to_string(),
                reconnect: true,
            },
            ErrorCode::UnknownOP => InvalidSessionData {
                code: *self,
                description: "Unknown OP code".to_string(),
                explanation: "The server
                received an unknown OP code. Try reconnecting?"
                    .to_string(),
                reconnect: true,
            },
            ErrorCode::DecodeError => InvalidSessionData {
                code: *self,
                description: "Decode error".to_string(),
                explanation: "The server
                received an invalid payload. Try reconnecting?"
                    .to_string(),
                reconnect: true,
            },
            ErrorCode::AuthenticationFailed => InvalidSessionData {
                code: *self,
                description: "Authentication failed".to_string(),
                explanation: "The server
                received an invalid passphrase too many times."
                    .to_string(),
                reconnect: false,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InvalidSessionData {
    code: ErrorCode,
    description: String,
    explanation: String,
    reconnect: bool,
}

// Hello data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HelloData {
    pub heartbeat_interval: u8,
}

// WebSocket payload data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PayloadData {
    Dispatch(DispatchData),
    Heartbeat,
    Identify(IdentifyData),
    ReIdentify,
    InvalidSession(InvalidSessionData),
    Hello(HelloData),
    HeartbeatAck,
}
