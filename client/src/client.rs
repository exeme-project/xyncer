use std::future::Future;

use bytes;
use fastwebsockets;
use http_body_util;
use hyper;
use hyper_util;
use std::sync::Arc;
use tokio;
use tokio::sync::RwLock;

use crate::session;
use xyncer_share::{self, Websocket};

// Tie Hyper's executor to Tokio's runtime
struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    // Spawns a future onto the Tokio runtime
    fn execute(&self, fut: Fut) {
        tokio::task::spawn(fut);
    }
}

// Connects to the specified WebSocket server
async fn connect(
    server_url: &str,
) -> Result<
    fastwebsockets::FragmentCollector<hyper_util::rt::TokioIo<hyper::upgrade::Upgraded>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    // Connect to the WebSocket server
    let stream = tokio::net::TcpStream::connect(server_url).await?;

    // Create a WebSocket handshake request
    let request = hyper::Request::builder()
        .uri(format!("ws://{}/", server_url))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            fastwebsockets::handshake::generate_key(),
        )
        .body(http_body_util::Empty::<bytes::Bytes>::new())?;

    // Perform the WebSocket handshake
    let (ws, _) = fastwebsockets::handshake::client(&SpawnExecutor, request, stream).await?;

    // Create a FragmentCollector to handle the WebSocket messages
    Ok(fastwebsockets::FragmentCollector::new(ws))
}

pub async fn start_client(
    session_data_guard: Arc<RwLock<session::Session>>,
    payload_receiver: flume::Receiver<xyncer_share::Payload>,
    payload_sender: flume::Sender<xyncer_share::Payload>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Obtain a read lock on the session data
    let session_data = session_data_guard.read().await;

    // Connect to the WebSocket server
    let mut websocket = connect(&session_data.server_address).await?;

    // Drop the read lock on the session data
    drop(session_data);

    // Obtain a write lock on the session data
    let mut session_data = session_data_guard.write().await;

    // Update the session data
    session_data.connected = true;
    session_data.error = None;

    // Drop the write lock on the session data
    drop(session_data);

    loop {
        tokio::select! {
                // Handle incoming WebSocket messages
                msg = websocket.read_frame() => {
                    let msg = msg.map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)?;

                    match msg.opcode {
                        fastwebsockets::OpCode::Binary => {
                            let bytes = msg.payload.to_owned();
                            let payload: xyncer_share::Payload = rmp_serde::from_slice(&bytes).unwrap();

                            log::info!("Received payload: {:?}", payload);

                            match payload.op_code {
                                // Start the heartbeat sender
                                xyncer_share::OP::Hello => {
                                    let heartbeat_interval = match payload.data {
                                        xyncer_share::payloads::PayloadData::Hello(data) => {
                                            data.heartbeat_interval
                                        }
                                        _ => {
                                            unimplemented!();
                                        }
                                    };

                                    let payload_sender_cloned = payload_sender.clone();
                                    let session_data_guard_cloned = session_data_guard.clone();

                                    tokio::spawn(async move {
                                        loop {
                                            // Obtain a read lock on the session data
                                            let session_data = session_data_guard_cloned.read().await;

                                            if !session_data.connected {
                                                log::info!("Stopping heartbeat (not connected)");

                                                break;
                                            }

                                            // Drop the read lock on the session data, ASAP to avoid deadlocks
                                            drop(session_data);

                                            if let Err(e) = payload_sender_cloned.send(xyncer_share::Payload {
                                                op_code: xyncer_share::OP::Heartbeat,
                                                event_name: xyncer_share::Event::None,
                                                data: xyncer_share::payloads::PayloadData::Heartbeat,
                                            }) {
                                                log::error!("Error sending heartbeat (stopping): {}", e);

                                                break;
                                            }

                                            tokio::time::sleep(tokio::time::Duration::from_secs(heartbeat_interval.into())).await;
                                        }
                                    });
                                },
                                // Server requesting a heartbeat
                                xyncer_share::OP::Heartbeat => {
                                    if let Err(e) = payload_sender.send(xyncer_share::Payload {
                                        op_code: xyncer_share::OP::Heartbeat,
                                        event_name: xyncer_share::Event::None,
                                        data: xyncer_share::payloads::PayloadData::Heartbeat,
                                    }) {
                                        log::error!("Error sending heartbeat in response to heartbeat request: {}", e);
                                    }
                                },
                                xyncer_share::OP::HeartbeatAck => {},
                                _ => {
                                    unimplemented!();
                                }
                            }
                        }
                        fastwebsockets::OpCode::Close => {
                            break;
                        }
                        _ => unimplemented!(),
                    }
                },
                // Handle outgoing WebSocket messages
                payload_result = payload_receiver.recv_async() => {
                    match payload_result {
                        Ok(payload) => {
                            if let Err(e) = websocket.send_payload(payload).await {
                                log::error!("Error sending WebSocket payload: {}", e);
                            }
                        },
                        Err(e) => {
                            log::error!("WebSocket payload sender channel error: {}", e);
                        },
                    }
                },
        }
    }

    Ok(())
}
