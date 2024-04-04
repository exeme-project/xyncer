use axum;
use fastwebsockets;
use rand::distributions::{Alphanumeric, DistString};
use std::net::SocketAddr;
use tokio;

use crate::session;
use xyncer_share::{self, Websocket};

// Run the WebSocket server
pub async fn start_server(ip: &str, port: u16) -> Result<(), std::io::Error> {
    let url = format!("{}:{}", ip, port);

    // Create a new router
    let app = axum::Router::new().route("/", axum::routing::get(upgrade_connection));

    // Bind the server to the address and port
    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();

    log::info!("WebSocket server running on ws://{}", url);

    // Start the server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}

// Handles a WebSocket connection
async fn handle_connection(
    future: fastwebsockets::upgrade::UpgradeFut,
    session_data: session::Session,
) -> Result<(), fastwebsockets::WebSocketError> {
    // Create a new WebSocket connection
    let mut websocket = fastwebsockets::FragmentCollector::new(future.await?);

    websocket
        .send_payload(xyncer_share::Payload {
            op_code: xyncer_share::OP::Hello,
            event_name: xyncer_share::Event::None,
            data: xyncer_share::payloads::PayloadData::Hello(xyncer_share::payloads::HelloData {
                heartbeat_interval: 60,
            }),
        })
        .await?;

    loop {
        // Read a frame from the WebSocket connection
        let msg = websocket.read_frame().await?;

        match msg.opcode {
            fastwebsockets::OpCode::Binary => {
                let bytes = msg.payload.to_owned();
                let payload: xyncer_share::Payload = rmp_serde::from_slice(&bytes).unwrap();

                log::info!("Received payload: {:?}", payload);
            }
            fastwebsockets::OpCode::Text => {
                unimplemented!();
            }
            fastwebsockets::OpCode::Close => break,
            _ => {}
        }
    }

    Ok(())
}

// Upgrades an HTTP connection to a WebSocket connection
async fn upgrade_connection(
    ws: fastwebsockets::upgrade::IncomingUpgrade,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
) -> impl axum::response::IntoResponse {
    // Upgrade the connection to a WebSocket connection
    let (response, future) = ws.upgrade().unwrap();

    let session_data = session::Session {
        authenticated: false,
        address: addr.to_string(),
        password: Alphanumeric.sample_string(&mut rand::thread_rng(), 5),
        password_attempts: 0,
    };

    log::info!("WebSocket connection established with: {}", addr);

    // Spawn a new task to handle the WebSocket connection
    tokio::task::spawn(async move {
        // Handle the WebSocket connection, and log any errors
        if let Err(e) = handle_connection(future, session_data).await {
            log::error!("WebSocket connection with {} failed: {}", addr, e);
        }
    });

    response
}
