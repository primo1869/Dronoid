#![forbid(unsafe_code)]

use futures::StreamExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error")]
    Error,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn run(port: u16) -> Result<()> {
    let (_, rx) = tokio::sync::oneshot::channel::<()>();

    let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|_err| Error::Error)?;

    run_custom(rx, tcp_listener).await?;
    Ok(())
}

pub async fn run_custom(
    mut stopper: tokio::sync::oneshot::Receiver<()>,
    tcp_listener: tokio::net::TcpListener,
) -> Result<()> {
    loop {
        tokio::select! {
            _ = &mut stopper => {
                return Ok(());
            }
            Ok((tcp_stream, _addr)) = tcp_listener.accept() => {
                tokio::spawn(async {process_client(tcp_stream).await});
            }
        }
    }
}
async fn process_client(tcp_stream: TcpStream) {
    let maybe_websocket = tokio_tungstenite::accept_async(tcp_stream).await;
    if maybe_websocket.is_err() {
        log::info!("Upgrade to websocket failed");
        return;
    }
    let mut websocket = maybe_websocket.unwrap();
    loop {
        let maybe_maybe_msg = websocket.next().await;
        if maybe_maybe_msg.is_none() {
            log::info!("No message");
            return;
        }
        let maybe_msg = maybe_maybe_msg.unwrap();
        if maybe_msg.is_err() {
            log::info!("Message error");
            return;
        }
        let msg = maybe_msg.unwrap();
        if !process_msg(msg).await {
            return;
        }
    }
}

async fn process_msg(msg: tungstenite::Message) -> bool {
    match msg {
        tungstenite::Message::Text(text) => {
            log::info!("Got text: {}", text);
            true
        }
        tungstenite::Message::Close(_reason) => {
            log::info!("Got close");
            false
        }
        _ => false,
    }
}
