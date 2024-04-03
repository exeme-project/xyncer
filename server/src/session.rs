pub struct Session {
    pub authenticated: bool,
    pub address: String,
    pub password: String,
    pub password_attempts: u8,
}
