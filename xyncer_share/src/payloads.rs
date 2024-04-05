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
pub enum ErrorCode {
    UnknownError,
    UnknownOP,
    DecodeError,
    AuthenticationFailed,
    SessionTimeout,
}

impl ErrorCode {
    pub fn populate(&self) -> InvalidSessionData {
        match self {
            ErrorCode::UnknownError => InvalidSessionData {
                code: *self,
                description: "Unknown error".to_string(),
                explanation: "We're not sure what went wrong. Try reconnecting?".to_string(),
            },
            ErrorCode::UnknownOP => InvalidSessionData {
                code: *self,
                description: "Unknown OP code".to_string(),
                explanation: "The server
                received an unknown OP code. Try reconnecting?"
                    .to_string(),
            },
            ErrorCode::DecodeError => InvalidSessionData {
                code: *self,
                description: "Decode error".to_string(),
                explanation: "The server
                received an invalid payload. Try reconnecting?"
                    .to_string(),
            },
            ErrorCode::AuthenticationFailed => InvalidSessionData {
                code: *self,
                description: "Authentication failed".to_string(),
                explanation: "The server
                received an invalid passphrase too many times."
                    .to_string(),
            },
            ErrorCode::SessionTimeout => InvalidSessionData {
                code: *self,
                description: "Session timeout".to_string(),
                explanation: "You didn't send a heartbeat in time.".to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InvalidSessionData {
    code: ErrorCode,
    description: String,
    explanation: String,
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
