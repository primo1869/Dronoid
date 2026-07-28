use std::collections::HashMap;
use std::net::SocketAddr;

use crossbeam_channel::{Receiver, Sender};

use crate::Error;
use crate::Result;
use crate::protocol;

pub struct OnlinePlayer {
    pub messages: Vec<protocol::ServerMessage>,
    message_sender: Sender<protocol::ServerMessage>,
    pub action_receiver: Receiver<protocol::Action>,
    pub _name: String,
    pub minerals_cnt: u32,
    pub owned_factories: HashMap<u32, bevy_ecs::entity::Entity>,
    pub to_kick: bool,
}

impl OnlinePlayer {
    pub fn new(
        message_sender: Sender<protocol::ServerMessage>,
        action_receiver: Receiver<protocol::Action>,
        name: String,
        minerals_cnt: u32,
    ) -> Self {
        Self {
            _name: name,
            action_receiver,
            message_sender,
            messages: Vec::new(),
            minerals_cnt,
            owned_factories: HashMap::new(),
            to_kick: false,
        }
    }
    pub fn flush_messages(&mut self) -> Result<()> {
        for message in self.messages.drain(0..) {
            self.message_sender
                .send(message)
                .map_err(|_| Error::FlushError)?;
        }
        Ok(())
    }
}

pub struct EnteringPlayer {
    pub message_sender: Sender<protocol::ServerMessage>,
    pub action_receiver: Receiver<protocol::Action>,
    pub id: u32,
    pub name: String,
    pub spawn_point: (f32, f32),
    pub addr: SocketAddr,
}
