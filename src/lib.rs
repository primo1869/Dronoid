#![forbid(unsafe_code)]

pub(crate) mod building;
pub(crate) mod drone;
pub(crate) mod game;
pub(crate) mod network;
pub(crate) mod player;
pub(crate) mod program;
pub(crate) mod utils;

pub mod protocol;

use crate::player::Player;

const TICK_DURATION: f64 = 1.0 / 60.0;

pub async fn run(stopper_rx: crossbeam_channel::Receiver<()>, port: u16) -> std::io::Result<()> {
    let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    run_with_listener(stopper_rx, tcp_listener).await;
    Ok(())
}

pub async fn run_with_listener(stopper_rx: crossbeam_channel::Receiver<()>, tcp_listener: tokio::net::TcpListener) {
    let (player_tx, player_rx) = crossbeam_channel::bounded::<Player>(100);
    let network_process_hdl = tokio::spawn(network::process(tcp_listener, player_tx));
    game::process(stopper_rx, player_rx);
    network_process_hdl.abort();
}
