use std::net::SocketAddr;

use crate::protocol::{PlayerAction, ServerMessage};

pub(crate) struct Player {
    pub(crate) sender: tokio::sync::mpsc::Sender<(bool, ServerMessage)>,
    pub(crate) receiver: tokio::sync::mpsc::Receiver<PlayerAction>,
    pub(crate) _spawn_point: (f32, f32, f32),
    pub(crate) authenticated: bool,
    pub(crate) name: String,
    pub(crate) _addr: SocketAddr,
}

impl Player {
    pub(crate) fn new(
        _addr: SocketAddr,
        sender: tokio::sync::mpsc::Sender<(bool, ServerMessage)>,
        receiver: tokio::sync::mpsc::Receiver<PlayerAction>,
    ) -> Player {
        Player {
            _addr,
            sender,
            receiver,
            _spawn_point: (0., 0., 0.),
            authenticated: false,
            name: String::default(),
        }
    }
}
