use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    Result,
    error::Error,
    game, network,
    player::Player,
    protocol::{PlayerAction, ServerMessage},
};

pub async fn run(port: u16) -> Result<()> {
    let (_sd, rx) = tokio::sync::oneshot::channel::<()>();

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
    let players = Arc::new(Mutex::new(Vec::<Player>::new()));
    let players_for_game = Arc::clone(&players);
    tokio::spawn(async move {
        game::main_loop(players_for_game).await;
    });

    log::info!(
        "Server listenning on port {}...",
        tcp_listener.local_addr().unwrap().port()
    );

    loop {
        let players_for_client = Arc::clone(&players);
        tokio::select! {
            _ = &mut stopper => {
                log::info!("Received stop signal!");
                return Ok(());
            }
            Ok((tcp_stream, addr)) = tcp_listener.accept() => {
                tokio::spawn(async move {
                    log::trace!("New connection from {}", addr);
                    let maybe_websocket = tokio_tungstenite::accept_async(tcp_stream).await;
                    if maybe_websocket.is_err() {
                        log::info!("Upgrade to websocket failed");
                        return;
                    }
                    let websocket = maybe_websocket.unwrap();
                    let (network_sender, loop_receiver) = tokio::sync::mpsc::channel::<PlayerAction>(1000);
                    let (loop_sender, network_receiver) = tokio::sync::mpsc::channel::<(bool, ServerMessage)>(1000);
                    players_for_client.lock().await.push(Player::new(addr, loop_sender, loop_receiver));
                    network::process(websocket, network_sender, network_receiver).await;
                });
            }
        }
    }
}
