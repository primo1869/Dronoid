#![forbid(unsafe_code)]

use crossbeam_channel::TryRecvError::Disconnected;
use crossbeam_channel::{Receiver, Sender};
use std::io;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite;

use crate::persistence::Database;
use crate::player::EnteringPlayer;

pub mod game;
pub mod persistence;
pub mod player;
pub mod transport;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network could not be initialized: {0}")]
    NetworkInitError(io::Error),
    #[error("Did not stop properly: {0}")]
    StopError(u8),
    #[error("Could not flush state to transport")]
    FlushError,
    #[error("Failed to join session: {0}")]
    WaitError(tokio::task::JoinError),
    #[error("No message received or it was of an unexpected type")]
    UnexpectedOrNoMessage,
    #[error("Client could not connect: {0}")]
    ClientConnectError(tungstenite::Error),
    #[error("Client could not authentication: {0}")]
    ClientFailedAuthentication(String),
    #[error("Client could not send data: {0}")]
    ClientSendError(tungstenite::Error),
    #[error("Client read error")]
    ClientReadError,
    #[error("Client could not receive data: {0}")]
    RecvError(tungstenite::Error),
    #[error("Stopper channel failed: {0}")]
    StopperError(crossbeam_channel::SendError<Stop>),
    #[error("Unexpected error: {0}")]
    UnexpectedError(&'static str),
    #[error("Transport error")]
    TransportError,
    #[error("Not an action")]
    NotAnAction,
    #[error("Unknown error")]
    UnknownError,
    #[error("ser/de failed: {0}")]
    Serde(bson::error::Error),
}

#[derive(Clone, Debug)]
pub struct ZoneExtensions {
    pub factory: f32,
}

impl Default for ZoneExtensions {
    fn default() -> Self {
        Self {
            factory: dronoid::defaults::zone_extensions::FACTORY,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub terrain_scale: f32,
    pub mineral_threshold: f32,
    pub tick_duration: f32,
    pub starting_minerals: u32,
    pub terrain_seed: u32,
    pub zone_extensions: ZoneExtensions,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            terrain_scale: dronoid::defaults::TERRAIN_SCALE,
            mineral_threshold: dronoid::defaults::MINERAL_THRESHOLD,
            tick_duration: dronoid::defaults::TICK_DURATION,
            starting_minerals: dronoid::defaults::STARTING_MINERALS,
            terrain_seed: dronoid::defaults::TERRAIN_SEED,
            zone_extensions: ZoneExtensions::default(),
        }
    }
}

pub enum Stop {
    Normal,
}

pub struct Commands {
    stopper: Sender<Stop>,
}

impl Commands {
    pub fn stop(&self) -> Result<()> {
        self.stopper
            .send(Stop::Normal)
            .map_err(Error::StopperError)?;
        Ok(())
    }
}

pub struct Controls {
    stopper: Receiver<Stop>,
}

impl Controls {
    pub fn stopped(&self) -> bool {
        match self.stopper.try_recv() {
            Err(Disconnected) => true,
            Ok(_) => true,
            _ => false,
        }
    }
}

pub async fn run(
    rules: Rules,
    database: Database,
    tcp_listener: TcpListener,
    controls: Controls,
) -> Result<()> {
    let (player_tx, player_rx) = crossbeam_channel::bounded::<EnteringPlayer>(1000);
    let (transport_stopper_tx, transport_stopper_rx) = crossbeam_channel::bounded::<()>(1);
    let transport_hdl = tokio::spawn(transport::run(
        database,
        tcp_listener,
        transport_stopper_rx,
        player_tx,
    ));
    let game_hdl = tokio::task::spawn_blocking(move || {
        game::run(rules, controls, transport_stopper_tx, player_rx)
    });
    let _ = tokio::join!(transport_hdl, game_hdl);
    Ok(())
}

pub fn new_commands() -> (Commands, Controls) {
    let (tx, rx) = crossbeam_channel::bounded(1);
    (Commands { stopper: tx }, Controls { stopper: rx })
}
