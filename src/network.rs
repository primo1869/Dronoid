use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio::{net::TcpStream, time};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::Result;
use crate::player::Player;
use crate::protocol::{PlayerAction, ServerMessage};

pub(crate) async fn process(
    mut stopper: tokio::sync::oneshot::Receiver<()>,
    tcp_listener: tokio::net::TcpListener,
    players: Arc<Mutex<Vec<Player>>>,
) -> Result<()> {
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
                    let (network_sender, loop_receiver) = crossbeam_channel::unbounded::<PlayerAction>();
                    let (loop_sender, network_receiver) = crossbeam_channel::unbounded::<(bool, ServerMessage)>();
                    players_for_client.lock().await.push(Player::new(addr, loop_sender, loop_receiver));
                    process_client(websocket, network_sender, network_receiver).await;
                });
            }
        }
    }
}

pub(crate) async fn process_client(mut websocket: WebSocketStream<TcpStream>, sender: Sender<PlayerAction>, receiver: Receiver<(bool, ServerMessage)>) {
    let mut interval = time::interval(Duration::from_secs_f32(1. / 60.));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Ok((keep, server_message)) = receiver.try_recv() {
                    let text = serde_json::to_string(&server_message).unwrap();
                    if websocket.send(Message::Text(text.into())).await.is_err() || !keep {
                        return;
                    }
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
                    if sender.send(maybe_client_message.unwrap()).is_err() {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
    }
}
