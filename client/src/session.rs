use anyhow::Error;

pub struct Session {
    pub authenticated: bool,
    pub connected: bool,
    pub error: Error,
    pub password: String,
    pub server_address: String,
}
