use simple_logger::SimpleLogger;

mod server;
mod session;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    if let Err(e) = server::start_server("0.0.0.0", 8080).await {
        log::error!("Error starting server: {}", e);
    }
}
