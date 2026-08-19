use std::net::SocketAddr;

use crate::protocol::{PlayerAction, ServerMessage};

pub(crate) struct Player {
    pub(crate) sender: tokio::sync::mpsc::Sender<(bool, ServerMessage)>,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<PlayerAction>,
    pub(crate) spawn_point: (f32, f32, f32),
    pub(crate) authenticated: bool,
    pub(crate) name: String,
    pub(crate) addr: SocketAddr,
}

impl Player {
    pub(crate) fn new(
        addr: SocketAddr,
        sender: tokio::sync::mpsc::Sender<(bool, ServerMessage)>,
        receiver: tokio::sync::mpsc::Receiver<PlayerAction>,
    ) -> Player {
        Player {
            addr,
            sender,
            receiver,
            spawn_point: (0., 0., 0.),
            authenticated: false,
            name: String::default(),
        }
    }
}
