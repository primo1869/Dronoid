use crate::Error;
use crate::persistence::{self, Database};
use crate::player::EnteringPlayer;
use crossbeam_channel::TryRecvError::{Disconnected, Empty};
use crossbeam_channel::{Receiver, Sender};
use dronoid::protocol::{
    Action, AuthenticationResponse, ClientMessage, Response, ServerMessage, is_name_valid,
};
use futures::{SinkExt, StreamExt};
use rapier2d::parry::utils::hashmap::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::tungstenite::Bytes;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use tracing::{debug, info, trace};

pub async fn run(
    database: Database,
    tcp_listener: TcpListener,
    transport_stopper_rx: Receiver<()>,
    client_tx: Sender<EnteringPlayer>,
) {
    let database_ref = Arc::new(Mutex::new(database));
    let players_ref = Arc::new(Mutex::new(HashMap::<String, ()>::default()));
    let mut interval = time::interval(Duration::from_secs_f32(1. / 100.));
    let local_addr = tcp_listener.local_addr().unwrap();
    info!(
        sender = "Transport",
        "Waiting for connection on {local_addr}"
    );
    loop {
        tokio::select! {
             _ = interval.tick() => {
                if let Err(Empty) = transport_stopper_rx.try_recv() {
                    continue;
                } else {
                    break;
                }
            }
            Ok((tcp_stream, addr)) = tcp_listener.accept() => {
                debug!(sender = addr.to_string(), "New TCP connection");
                let new_player_tx_cln = client_tx.clone();
                let database_ref = database_ref.clone();
                let players_ref_cln1 = players_ref.clone();
                let players_ref_cln2 = players_ref.clone();
                tokio::spawn(async move {
                    let maybe_websocket = tokio_tungstenite::accept_async::<MaybeTlsStream<TcpStream>>(MaybeTlsStream::Plain(tcp_stream)).await;
                    if maybe_websocket.is_err() {
                        let err = maybe_websocket.err().unwrap();
                        debug!(sender = addr.to_string(), "Websocket upgrade error: {err}");
                        return;
                    }
                    debug!(sender = addr.to_string(), "Upgraded to websocket");
                    let websocket = maybe_websocket.unwrap();

                    let (player_name, result) = serve_client(websocket, addr, new_player_tx_cln, database_ref, players_ref_cln1).await;
                    let player_name = if player_name.is_some() {
                        let player_name = player_name.unwrap();
                        players_ref_cln2.lock().await.remove(&player_name);
                        player_name
                    } else {
                        String::from("<unknown>")
                    };
                    match result {
                        Err(err) => {
                            info!(sender = addr.to_string(), "Played '{player_name}' kicked: {err}");
                        }
                        Ok(_) => {
                            info!(sender = addr.to_string(), "Player '{player_name}' left");
                        }
                    }
                });
            }
        }
    }
}

pub async fn serve_client(
    mut websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    addr: SocketAddr,
    new_player_tx: Sender<crate::player::EnteringPlayer>,
    database_ref: Arc<Mutex<Database>>,
    players_ref: Arc<Mutex<HashMap<String, ()>>>,
) -> (Option<String>, crate::Result<()>) {
    let (action_sender, action_receiver) = crossbeam_channel::bounded::<Action>(100);
    let (state_sender, state_receiver) = crossbeam_channel::bounded::<ServerMessage>(100);
    let player_name: String;
    if let Some(Ok(Message::Binary(binary))) = websocket.next().await {
        debug!(sender = addr.to_string(), "Authentication request received");
        let maybe_authentication =
            bson::deserialize_from_slice::<ClientMessage>(binary.iter().as_slice())
                .map_err(Error::Serde);
        if maybe_authentication.is_err() {
            return (None, Err(Error::UnknownError));
        }
        match maybe_authentication.unwrap() {
            ClientMessage::AuthenticationRequest(request) => {
                player_name = request.player_name;
            }
            _ => {
                return (None, Err(Error::UnknownError));
            }
        }

        let mut database_guard = database_ref.lock().await;
        let (maybe_player_entry, response) = if players_ref.lock().await.contains_key(&player_name)
        {
            (
                None,
                ServerMessage::Response(Response::AuthenticationResponse(
                    AuthenticationResponse::already_playing(),
                )),
            )
        } else {
            authentication(addr, player_name.clone(), &mut database_guard).await
        };

        let response_text = bson::serialize_to_vec(&response).unwrap();
        if websocket
            .send(Message::Binary(response_text.into()))
            .await
            .is_err()
        {
            return (Some(player_name), Err(Error::UnknownError));
        }
        if let Some(player_entry) = maybe_player_entry.clone() {
            if new_player_tx
                .send(EnteringPlayer {
                    message_sender: state_sender,
                    action_receiver,
                    id: player_entry.id,
                    name: player_entry.name,
                    spawn_point: player_entry.spawn_point,
                    addr,
                })
                .is_err()
            {
                return (Some(player_name), Err(Error::UnknownError));
            }
            if maybe_player_entry.is_some() {}
        } else {
            return (Some(player_name), Err(Error::UnknownError));
        }
    } else {
        return (None, Err(Error::UnknownError));
    }

    players_ref.lock().await.insert(player_name.clone(), ());
    let mut interval = time::interval(Duration::from_secs_f32(1. / 10.));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let result = loop {
                    match state_receiver.try_recv() {
                        Ok(message) => {
                            let binary = Bytes::from(bson::serialize_to_vec(&message).unwrap());
                            if websocket.send(Message::Binary(binary)).await.is_err()  {
                                break Err(Error::TransportError)
                            }
                        }
                        Err(Disconnected) => {
                            break Err(Error::UnexpectedError("Recv from game loop disconnected !"))
                        }
                        _ => {
                            break Ok(())
                        }
                    }
                };
                if result.is_err() {
                    return (Some(player_name), Err(result.err().unwrap()));
                }
            }

            Some(maybe_msg) = websocket.next() => {
                if maybe_msg.is_err() {
                    return (Some(player_name), Err(Error::TransportError));
                }
                let message = maybe_msg.unwrap();
                if let Message::Binary(bin) = message {
                    trace!(sender = addr.to_string(), "Received action");
                    let maybe_action = bson::deserialize_from_slice::<Action>(bin.iter().as_slice());

                    if maybe_action.is_err() {
                        return (Some(player_name), Err(Error::NotAnAction));
                    }
                    if action_sender.send(maybe_action.unwrap()).is_err() {
                        return (Some(player_name), Err(Error::UnexpectedError("Could not send to game loop !")));
                    }
                } else if let Message::Close(_) = message {
                    return (Some(player_name), Ok(()))
                }
                else {
                    return (Some(player_name), Err(Error::UnexpectedOrNoMessage));
                }
            }
        }
    }
}

async fn authentication(
    addr: SocketAddr,
    player_name: String,
    database_guard: &mut tokio::sync::MutexGuard<'_, Database>,
) -> (Option<persistence::PlayerEntry>, ServerMessage) {
    if let Some(player_entry) = database_guard.get_player_entry(&player_name).await {
        let spawn_point = player_entry.spawn_point;
        (
            Some(player_entry),
            ServerMessage::Response(Response::AuthenticationResponse(
                AuthenticationResponse::welcome(spawn_point),
            )),
        )
    } else {
        if !is_name_valid(&player_name) {
            (
                None,
                ServerMessage::Response(Response::AuthenticationResponse(
                    AuthenticationResponse::invalid_name(),
                )),
            )
        } else {
            let spawn_x = rand::random_range(-10000_f32..10000_f32);
            let spawn_y = rand::random_range(-10000_f32..10000_f32);
            let spawn_point = (spawn_x, spawn_y);
            info!(
                sender = addr.to_string(),
                "New player '{player_name}' spawned at {spawn_x} {spawn_y}"
            );
            let player_entry = database_guard
                .add_player_entry(player_name, spawn_point)
                .await;
            (
                Some(player_entry),
                ServerMessage::Response(Response::AuthenticationResponse(
                    AuthenticationResponse::welcome(spawn_point),
                )),
            )
        }
    }
}
