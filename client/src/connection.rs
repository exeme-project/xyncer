use std::future::Future;

use anyhow;
use bytes;
use fastwebsockets;
use http_body_util;
use hyper;
use hyper_util;
use tokio;

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
    url: &str,
) -> anyhow::Result<
    fastwebsockets::FragmentCollector<hyper_util::rt::TokioIo<hyper::upgrade::Upgraded>>,
> {
    // Connect to the WebSocket server
    let stream = tokio::net::TcpStream::connect(url).await?;

    // Create a WebSocket handshake request
    let request = hyper::Request::builder()
        .uri(format!("ws://{}/", url))
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
