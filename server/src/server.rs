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

    let mut last_heartbeat = tokio::time::Instant::now();
    let mut requested_heartbeat_from_client = false;

    loop {
        let mut sleep_duration = tokio::time::Duration::from_secs(0);

        // Make sure we have not gone past the waiting time (5 more seconds as a jitter)
        if last_heartbeat.elapsed() <= tokio::time::Duration::from_secs(65) {
            // If we haven't gone past the waiting time, calculate the remaining time
            sleep_duration = tokio::time::Duration::from_secs(65) - last_heartbeat.elapsed();
        }

        tokio::select! {
            // Check if we have not received a heartbeat
            _ = tokio::time::sleep(sleep_duration) => {
                if requested_heartbeat_from_client {
                    // Close the connection because the client did not respond to the heartbeat request
                    log::warn!("Client did not respond to heartbeat request, closing connection");

                    websocket
                        .send_payload(xyncer_share::Payload {
                            op_code: xyncer_share::OP::InvalidSession,
                            event_name: xyncer_share::Event::None,
                            data: xyncer_share::payloads::PayloadData::InvalidSession(xyncer_share::payloads::ErrorCode::SessionTimeout.populate()),
                        })
                        .await?;

                    websocket.close().await?;

                    break;
                } else {
                    // Request a heartbeat from the client
                    requested_heartbeat_from_client = true;

                    // Give the client a grace period to respond to the heartbeat request
                    last_heartbeat = tokio::time::Instant::now();

                    websocket
                        .send_payload(xyncer_share::Payload {
                            op_code: xyncer_share::OP::Heartbeat,
                            event_name: xyncer_share::Event::None,
                            data: xyncer_share::payloads::PayloadData::Heartbeat,
                        })
                        .await?;
                }
            }
            // Check for an incoming message
            msg = websocket.read_frame() => {
                let msg = msg?;

                match msg.opcode {
                    fastwebsockets::OpCode::Binary => {
                        let bytes = msg.payload.to_owned();
                        let payload: xyncer_share::Payload = rmp_serde::from_slice(&bytes).unwrap();

                        log::info!("Received payload: {:?}", payload);

                        match payload.op_code {
                            xyncer_share::OP::Heartbeat => {
                                last_heartbeat = tokio::time::Instant::now();
                                requested_heartbeat_from_client = false;

                                websocket
                                    .send_payload(xyncer_share::Payload {
                                        op_code: xyncer_share::OP::HeartbeatAck,
                                        event_name: xyncer_share::Event::None,
                                        data: xyncer_share::payloads::PayloadData::HeartbeatAck,
                                    })
                                    .await?;
                            }
                            _ => {
                                unimplemented!()
                            }
                        }
                    }
                    fastwebsockets::OpCode::Text => {
                        unimplemented!();
                    }
                    fastwebsockets::OpCode::Close => break,
                    _ => {}
                }
            }
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
