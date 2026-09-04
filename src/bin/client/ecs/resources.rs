use bevy::{asset::Handle, image::Image};
use bevy_ecs::{entity::Entity, resource::Resource};
use dronoid::protocol::Kind;
use std::{collections::HashMap, net::TcpStream};
use tokio_tungstenite::tungstenite::{self, stream::MaybeTlsStream};

#[derive(Resource, Default)]
pub struct SpawnPoint(pub (f32, f32));

#[derive(Resource, Default)]
pub struct Entities(pub HashMap<u32, Entity>);

#[derive(Resource, Default)]
pub struct GameSprites(pub HashMap<Kind, (f32, Handle<Image>)>);

#[derive(Resource, PartialEq)]
pub enum State {
    SetupConnectPage,
    ShowConnectPage,
    Connect,
    ShowGame,
}

impl Default for State {
    fn default() -> Self {
        State::SetupConnectPage
    }
}

#[derive(Resource)]
pub struct PlayerCredentials {
    pub name: String,
}

#[derive(Resource)]
pub struct ServerHost(pub String);

#[derive(Resource)]
pub struct ServerPort(pub u16);

#[derive(Resource, Default)]
pub struct CurrentDisplayResolution(pub (u32, u32));

#[derive(Resource)]
pub struct Connection(pub tungstenite::WebSocket<MaybeTlsStream<TcpStream>>);

impl Connection {
    pub fn new(websocket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        Self { 0: websocket }
    }
}
