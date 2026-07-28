#![forbid(unsafe_code)]

pub mod game;
pub mod persistence;
pub mod player;
pub mod protocol;
pub mod transport;

use colored::{Color, Colorize};
use crossbeam_channel::TryRecvError::Disconnected;
use crossbeam_channel::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use std::io;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, info};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

use crate::persistence::Database;
use crate::player::EnteringPlayer;

pub type Result<T> = std::result::Result<T, Error>;

// #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
// pub struct u32(u32);
// impl u32 {
//     fn new() -> Self {
//         Self {
//             0: random_range(u32::MIN..u32::MAX),
//         }
//     }
// }

// pub type (f32, f32) = (f32, f32);

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

pub mod defaults {
    pub const PORT: u16 = 2468;
    pub const TERRAIN_SCALE: f32 = 0.05;
    pub const MINERAL_THRESHOLD: f32 = 0.6;
    pub const TICK_DURATION: f32 = 1. / 10.;
    pub const STARTING_MINERALS: u32 = 100;
    pub const TERRAIN_SEED: u32 = 0;
    pub mod zone_extensions {
        pub const FACTORY: f32 = 60.;
    }
}

#[derive(Clone, Debug)]
pub struct ZoneExtensions {
    pub factory: f32,
}

impl Default for ZoneExtensions {
    fn default() -> Self {
        Self {
            factory: defaults::zone_extensions::FACTORY,
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
            terrain_scale: defaults::TERRAIN_SCALE,
            mineral_threshold: defaults::MINERAL_THRESHOLD,
            tick_duration: defaults::TICK_DURATION,
            starting_minerals: defaults::STARTING_MINERALS,
            terrain_seed: defaults::TERRAIN_SEED,
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
            .map_err(|err| Error::StopperError(err))?;
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

#[derive(Default)]
struct MessageVisitor {
    message: String,
    sender: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "sender" {
            self.sender = value.to_string();
        }
    }
}

struct CustomLayer {
    time_mark: Instant,
}

impl CustomLayer {
    fn new() -> Self {
        Self {
            time_mark: Instant::now(),
        }
    }
}

impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let color = match event.metadata().level() {
            &Level::TRACE => Color::BrightBlack,
            &Level::DEBUG => Color::Green,
            &Level::INFO => Color::White,
            &Level::WARN => Color::Yellow,
            &Level::ERROR => Color::Red,
        };

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let formatted = format!(
            "[{:>7.3}][{:^15}]{}",
            (Instant::now() - self.time_mark).as_secs_f32(),
            visitor.sender,
            visitor.message
        );
        println!("{}", formatted.color(color));
    }
}

pub fn init_logger() {
    let _ = Registry::default()
        .with(CustomLayer::new())
        .with(EnvFilter::from_default_env())
        .try_init();
}

pub struct Client {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub spawn_point: (f32, f32),
}

impl Client {
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        info!(sender = "Client", "Connecting to {addr}");
        let (stream, _) = tokio_tungstenite::connect_async(
            format!("ws://{}:{}", addr.ip(), addr.port()).as_str(),
        )
        .await
        .map_err(|err| Error::ClientConnectError(err))?;
        Ok(Self {
            stream,
            spawn_point: (0., 0.),
        })
    }

    pub async fn authenticated(addr: SocketAddr, player_name: &'static str) -> Result<Self> {
        let mut client = Self::connect(addr).await?;
        let response = client.authenticate(player_name).await?;
        if !response.result {
            return Err(Error::ClientFailedAuthentication(response.text));
        }
        Ok(client)
    }

    pub async fn authenticate(
        &mut self,
        player_name: &'static str,
    ) -> Result<protocol::AuthenticationResponse> {
        info!(sender = "Client", "Authenticating");
        self.stream
            .send(Message::Binary(
                protocol::serialize(&protocol::AuthenticationRequest {
                    player_name: player_name.to_string(),
                })
                .unwrap()
                .into(),
            ))
            .await
            .map_err(|err| Error::ClientSendError(err))?;
        if let Some(std::result::Result::Ok(Message::Binary(auth_resp_text))) =
            self.stream.next().await
        {
            if let std::result::Result::Ok(response) =
                protocol::deserialize::<protocol::AuthenticationResponse>(auth_resp_text)
            {
                self.spawn_point = response.spawn_point;
                return Ok(response);
            } else {
                return Err(Error::ClientReadError);
            }
        } else {
            return Err(Error::ClientReadError);
        }
    }

    pub async fn send_action(&mut self, action: protocol::Action) -> Result<()> {
        info!(sender = "Client", "Sending action");
        self.stream
            .send(Message::Binary(
                protocol::serialize(&action).unwrap().into(),
            ))
            .await
            .map_err(|err| Error::ClientSendError(err))?;
        Ok(())
    }

    pub async fn action(&mut self, action: protocol::Action) -> Result<protocol::Response> {
        self.send_action(action).await?;
        self.until_response().await
    }

    pub async fn next_message(&mut self) -> Result<protocol::ServerMessage> {
        let maybe_maybe_message = self.stream.next().await;
        if maybe_maybe_message.is_none() {
            return Err(Error::ClientReadError);
        }
        let message = maybe_maybe_message
            .unwrap()
            .map_err(|err| Error::RecvError(err))?;
        if let Message::Binary(text) = message {
            let server_message: protocol::ServerMessage =
                protocol::deserialize(text).map_err(|_| Error::ClientReadError)?;
            return Ok(server_message);
        } else {
            return Err(Error::ClientReadError);
        }
    }

    pub async fn until_response(&mut self) -> Result<protocol::Response> {
        loop {
            match self.next_message().await? {
                protocol::ServerMessage::State(_) => continue,
                protocol::ServerMessage::Response(response) => {
                    return Ok(response);
                }
            }
        }
    }

    pub async fn until_state(&mut self) -> Result<protocol::State> {
        loop {
            match self.next_message().await? {
                protocol::ServerMessage::State(state) => {
                    return Ok(state);
                }
                protocol::ServerMessage::Response(_) => continue,
            }
        }
    }
}
