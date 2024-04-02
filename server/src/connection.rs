use axum;
use fastwebsockets;
use tokio;

// Run the WebSocket server
async fn run() {
    // Create a new router
    let app = axum::Router::new().route("/", axum::routing::get(upgrade_connection));

    // Bind the server to the address and port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}

// Handles a WebSocket connection
async fn handle_connection(
    future: fastwebsockets::upgrade::UpgradeFut,
) -> Result<(), fastwebsockets::WebSocketError> {
    // Create a new WebSocket connection
    let mut ws = fastwebsockets::FragmentCollector::new(future.await?);

    loop {
        // Read a frame from the WebSocket connection
        let frame = ws.read_frame().await?;

        match frame.opcode {
            fastwebsockets::OpCode::Close => break,
            fastwebsockets::OpCode::Text | fastwebsockets::OpCode::Binary => {
                ws.write_frame(frame).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

// Upgrades an HTTP connection to a WebSocket connection
async fn upgrade_connection(
    ws: fastwebsockets::upgrade::IncomingUpgrade,
) -> impl axum::response::IntoResponse {
    // Upgrade the connection to a WebSocket connection
    let (response, future) = ws.upgrade().unwrap();

    // Spawn a new task to handle the WebSocket connection
    tokio::task::spawn(async move {
        // Handle the WebSocket connection, and log any errors
        if let Err(e) = handle_connection(future).await {
            eprintln!("Error in WebSocket connection: {}", e);
        }
    });

    response
}
