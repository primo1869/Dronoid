#![forbid(unsafe_code)]

use futures::SinkExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use thiserror::Error;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

#[derive(Serialize, Deserialize)]
pub enum PlayerAction {
    Authentication { player_name: String },
    PlaceFactory { pos_x: f32, pos_y: f32, pos_z: f32 },
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub result: bool,
    pub text: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    AuthenticationResponse(AuthenticationResponse),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error")]
    Error,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn run(port: u16) -> Result<()> {
    let (_sd, rx) = tokio::sync::oneshot::channel::<()>();

    let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|_err| Error::Error)?;

    run_custom(rx, tcp_listener).await?;
    Ok(())
}

struct Player {
    sender: tokio::sync::mpsc::Sender<(bool, ServerMessage)>,
    receiver: tokio::sync::mpsc::Receiver<PlayerAction>,
    #[allow(unused)]
    spawn_point: (f32, f32, f32),
    authenticated: bool,
    name: String,
    _addr: SocketAddr,
}

impl Player {
    fn new(
        _addr: SocketAddr,
        sender: tokio::sync::mpsc::Sender<(bool, ServerMessage)>,
        receiver: tokio::sync::mpsc::Receiver<PlayerAction>,
    ) -> Player {
        Player {
            _addr,
            sender,
            receiver,
            spawn_point: (0., 0., 0.),
            authenticated: false,
            name: String::default(),
        }
    }
}

async fn cycle(players: &mut Vec<Player>) {
    let mut idxs_to_remove = Vec::<usize>::new();
    let mut i = 0;
    let player_names: Vec<String> = players.iter().map(|player| player.name.clone()).collect();
    for player in &mut *players {
        let maybe_player_action = player.receiver.try_recv();
        match maybe_player_action {
            Err(err) => match err {
                tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                    idxs_to_remove.push(i);
                }
                tokio::sync::mpsc::error::TryRecvError::Empty => continue,
            },
            Ok(player_action) => match player_action {
                PlayerAction::Authentication { player_name } => {
                    let response: (bool, ServerMessage) = if player.authenticated {
                        log::trace!("Already authenticated");
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("Already authenticated").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    } else if !is_name_valid(&player_name) {
                        log::trace!("Invalid name {}", player_name);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("Invalid name").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    } else if player_names
                        .iter()
                        .find(|&other_name| other_name == &player_name)
                        .is_some()
                    {
                        log::trace!("Name already taken: {}", player_name);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("A player already has this name").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    } else {
                        log::trace!("Authenticated: {}", player_name);
                        player.authenticated = true;
                        player.name = player_name;
                        (
                            true,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: true,
                                text: String::from_str("Welcome").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    };
                    if player.sender.send(response).await.is_err() {
                        idxs_to_remove.push(i);
                    }
                }
                PlayerAction::PlaceFactory {
                    #[allow(unused)]
                    pos_x,
                    #[allow(unused)]
                    pos_y,
                    #[allow(unused)]
                    pos_z,
                } => {
                    continue;
                }
            },
        }
        i += 1;
    }

    for idx in idxs_to_remove.iter().rev() {
        players.remove(*idx);
    }
}

fn is_name_valid(player_name: &String) -> bool {
    if player_name.is_empty() { false } else { true }
}

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
        log::trace!("Main loop is running...");
        loop {
            cycle(players_for_loop.lock().await.as_mut()).await;
            let late_of = (tokio::time::Instant::now() - time_mark).as_secs_f32() / TICK_DURATION;
            if late_of > 1. {
                log::warn!("Server is late of {} tick(s)", late_of.trunc())
            }
            time_mark += tokio::time::Duration::from_secs_f32(late_of.ceil());

            tokio::time::sleep_until(time_mark).await;
        }
    });

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
                    process(websocket, network_sender, network_receiver).await;
                });
            }
        }
    }
}

async fn process(
    mut websocket: WebSocketStream<TcpStream>,
    sender: tokio::sync::mpsc::Sender<PlayerAction>,
    mut receiver: tokio::sync::mpsc::Receiver<(bool, ServerMessage)>,
) {
    loop {
        tokio::select! {
            Some((keep, server_message)) = receiver.recv() => {
                let text = serde_json::to_string(&server_message).unwrap();
                if websocket.send(Message::Text(text.into())).await.is_err() || !keep {
                    return;
                }
            }
            Some(maybe_msg) = websocket.next() => {
                if maybe_msg.is_err() {
                    log::trace!("Error on read");
                    return;
                }
                let message = maybe_msg.unwrap();
                if let Message::Text(text_message) = message {
                    let maybe_client_message = serde_json::from_str::<PlayerAction>(&text_message);
                    if maybe_client_message.is_err() {
                        return;
                    }
                    if sender.send(maybe_client_message.unwrap()).await.is_err() {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
    }
}
