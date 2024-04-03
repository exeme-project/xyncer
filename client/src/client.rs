use std::{future::Future, io::Read};

use anyhow;
use bytes;
use fastwebsockets;
use http_body_util;
use hyper;
use hyper_util;
use std::sync::{Arc, Mutex};
use tokio;

use crate::session;
use xyncer_share;

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
) -> anyhow::Result<
    fastwebsockets::FragmentCollector<hyper_util::rt::TokioIo<hyper::upgrade::Upgraded>>,
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
    server_url: &str,
    session: Arc<Mutex<session::Session>>,
) -> anyhow::Result<()> {
    let mut websocket = connect(server_url).await?;

    {
        let mut session_guard = session.lock().unwrap(); // Acquire the lock on the session.

        session_guard.connected = true;
        session_guard.error = anyhow::Error::msg("");
    } // The lock is automatically released here as session_guard goes out of scope.

    tokio::spawn(async move {
        loop {
            let msg = websocket.read_frame().await.map_err(anyhow::Error::new)?;

            match msg.opcode {
                fastwebsockets::OpCode::Binary => {
                    let bytes = msg.payload.to_owned();
                    let payload: xyncer_share::Payload = rmp_serde::from_slice(&bytes).unwrap();

                    log::info!("Received payload: {:?}", payload);

                    match payload.op_code {
                        xyncer_share::OP::Hello => {}
                        _ => {
                            unimplemented!();
                        }
                    }
                }
                fastwebsockets::OpCode::Text => {
                    unimplemented!();
                }
                fastwebsockets::OpCode::Close => {
                    break;
                }
                _ => {}
            }
        }

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
