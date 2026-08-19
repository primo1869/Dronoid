use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    Result, game, network,
    player::Player,
    protocol::{PlayerAction, ServerMessage},
};

const TICK_DURATION: f32 = 0.1;

pub async fn run_custom(
    mut stopper: tokio::sync::oneshot::Receiver<()>,
    tcp_listener: tokio::net::TcpListener,
) -> Result<()> {
    let players = Arc::new(Mutex::new(Vec::<Player>::new()));
    let start_time = tokio::time::Instant::now();
    let mut time_mark = start_time.clone();
    let players_for_loop = Arc::clone(&players);
    tokio::spawn(async move {
        log::info!("Main loop is running...");
        loop {
            game::cycle(players_for_loop.lock().await.as_mut()).await;
            let late_of = (tokio::time::Instant::now() - time_mark).as_secs_f32() / TICK_DURATION;
            if late_of > 1. {
                log::warn!("Server is late of {} tick(s)", late_of.trunc())
            }
            time_mark += tokio::time::Duration::from_secs_f32(late_of.ceil());

            tokio::time::sleep_until(time_mark).await;
        }
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
