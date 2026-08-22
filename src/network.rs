use crossbeam_channel::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::player::Player;
use crate::protocol::{PlayerAction, ServerMessage};

pub(crate) async fn process(tcp_listener: tokio::net::TcpListener, player_tx: Sender<Player>) {
    log::info!("Network loop is running");
    loop {
        let player_tx_cln = player_tx.clone();
        if let Ok((tcp_stream, addr)) = tcp_listener.accept().await {
            tokio::spawn(async move {
                let maybe_websocket = tokio_tungstenite::accept_async(tcp_stream).await;
                if maybe_websocket.is_err() {
                    return;
                }
                let websocket = maybe_websocket.unwrap();
                let (network_sender, loop_receiver) = crossbeam_channel::bounded::<PlayerAction>(100);
                let (loop_sender, network_receiver) = crossbeam_channel::bounded::<(bool, ServerMessage)>(100);
                if player_tx_cln.send(Player::new(addr, loop_sender, loop_receiver)).is_err() {
                    return;
                }

                process_client(websocket, network_sender, network_receiver).await;
            });
        }
    }
}

pub(crate) async fn process_client(
    mut websocket: WebSocketStream<TcpStream>,
    sender: Sender<PlayerAction>,
    receiver: Receiver<(bool, ServerMessage)>,
) {
    let mut interval = time::interval(Duration::from_secs_f32(1. / 60.));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                while let Ok((keep, server_message)) = receiver.try_recv() {
                    let text = serde_json::to_string(&server_message).unwrap();
                    if websocket.send(Message::Text(text.into())).await.is_err() || !keep {
                        return;
                    }
                }
            }
            Some(maybe_msg) = websocket.next() => {
                if maybe_msg.is_err() {

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
