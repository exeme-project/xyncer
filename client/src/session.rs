#[derive(Clone)]
pub struct Session {
    pub authenticated: bool,
    pub connected: bool,

    pub error: Option<String>,
    pub password: String,

    pub server_address: String,
}
