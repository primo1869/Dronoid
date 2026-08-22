use std::net::SocketAddr;

use crossbeam_channel::{Receiver, Sender};

use crate::protocol::{PlayerAction, ServerMessage};

pub(crate) struct Player {
    pub(crate) id: i64,
    pub(crate) sender: Sender<(bool, ServerMessage)>,
    pub(crate) receiver: Receiver<PlayerAction>,
    pub(crate) spawn_point: (f32, f32),
    pub(crate) authenticated: bool,
    pub(crate) name: String,
    pub(crate) _addr: SocketAddr,
}

impl Player {
    pub(crate) fn new(_addr: SocketAddr, sender: Sender<(bool, ServerMessage)>, receiver: Receiver<PlayerAction>) -> Player {
        Player {
            id: i64::default(),
            _addr,
            sender,
            receiver,
            spawn_point: (0., 0.),
            authenticated: false,
            name: String::default(),
        }
    }
}
