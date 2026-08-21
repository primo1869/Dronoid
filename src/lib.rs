#![forbid(unsafe_code)]

pub(crate) mod building;
pub(crate) mod drone;
pub(crate) mod error;
pub(crate) mod game;
pub(crate) mod network;
pub(crate) mod player;
pub(crate) mod program;
pub(crate) mod utils;

pub mod protocol;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{error::Error, player::Player};

pub type Result<T> = std::result::Result<T, Error>;

const TICK_DURATION: f64 = 1.0 / 60.0;

pub async fn run(port: u16) -> Result<()> {
    let (_sd, rx) = tokio::sync::oneshot::channel::<()>();

    let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|_err| Error::Error)?;

    run_custom(rx, tcp_listener).await?;
    Ok(())
}

pub async fn run_custom(stopper: tokio::sync::oneshot::Receiver<()>, tcp_listener: tokio::net::TcpListener) -> Result<()> {
    let players = Arc::new(Mutex::new(Vec::<Player>::new()));
    let players_for_game = Arc::clone(&players);
    let game_process_hdl = tokio::spawn(async move {
        game::process(players_for_game).await?;
        crate::Result::Ok(())
    });

    log::info!("Server listenning on port {}...", tcp_listener.local_addr().unwrap().port());

    let network_process_hdl = tokio::spawn(async move {
        network::process(stopper, tcp_listener, players).await?;
        crate::Result::Ok(())
    });

    let (res1, res2) = tokio::try_join!(game_process_hdl, network_process_hdl).map_err(|_err| Error::Error)?;
    res1?;
    res2?;

    Ok(())
}
