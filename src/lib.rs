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

pub async fn run(stopper_rx: crossbeam_channel::Receiver<()>, port: u16) -> Result<()> {
    let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|_err| Error::Error)?;

    run_with_listener(stopper_rx, tcp_listener).await?;
    Ok(())
}

pub async fn run_with_listener(stopper_rx: crossbeam_channel::Receiver<()>, tcp_listener: tokio::net::TcpListener) -> Result<()> {
    let players = Arc::new(Mutex::new(Vec::<Player>::new()));
    let players_for_game_loop = Arc::clone(&players);

    let game_process_hdl = tokio::spawn(async move {
        game::process(stopper_rx.clone(), players_for_game_loop);
    });

    let network_process_hdl = tokio::spawn(async move {
        network::process(stopper_rx, tcp_listener, players).await?;
        crate::Result::Ok(())
    });

    // game_process_hdl.abort();

    tokio::select! {
        result = game_process_hdl => {
            log::info!("Game loop interrupted");
            if result.is_err() {
                log::error!("{}", result.err().unwrap());
            }
        },
        result = network_process_hdl => {
            log::info!("Network loop interrupted");
            if result.is_err() {
                log::error!("{}", result.err().unwrap());
            }
        }
    };

    Ok(())
}
