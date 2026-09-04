use bevy_ecs::component::Component;
use std::net::TcpStream;
use tokio_tungstenite::tungstenite::WebSocket;

#[derive(Component)]
pub struct ServerConnection(pub WebSocket<TcpStream>);
