use crossbeam_channel::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::protocol::{PlayerAction, ServerMessage};

pub(crate) async fn process_client(
    mut websocket: WebSocketStream<TcpStream>,
    sender: Sender<PlayerAction>,
    mut receiver: Receiver<(bool, ServerMessage)>,
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
                    if sender.send(maybe_client_message.unwrap()).unwrap().is_err() {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
    }
}
