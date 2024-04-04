use std::future::Future;

use bytes;
use fastwebsockets;
use http_body_util;
use hyper;
use hyper_util;
use tokio;

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
    mut session_data: session::Session,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut websocket = connect(&session_data.server_address).await?;

    session_data.connected = true;
    session_data.error = None;

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

                                    let session_data = session_data.clone();

                                    tokio::spawn(async move {
                                        loop {
                                            tokio::time::sleep(tokio::time::Duration::from_secs(heartbeat_interval.into())).await;

                                            if session_data.connected {
                                                session_data.payload_sender.send(xyncer_share::Payload {
                                                    op_code: xyncer_share::OP::Heartbeat,
                                                    event_name: xyncer_share::Event::None,
                                                    data: xyncer_share::payloads::PayloadData::Heartbeat,
                                                }).unwrap();
                                            } else {
                                                break;
                                            }
                                        }
                                    });
                                }
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
                payload_result = session_data.payload_receiver.recv_async() => {
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
