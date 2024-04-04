use flume;
use simple_logger;

mod client;
mod session;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let (payload_sender, payload_receiver) = flume::unbounded();

    let session = session::Session {
        authenticated: false,
        connected: false,
        error: None,
        password: String::new(),
        payload_sender: payload_sender,
        payload_receiver: payload_receiver,
        server_address: String::from("127.0.0.1:8080"),
    };

    client::start_client(session).await.unwrap();
}
