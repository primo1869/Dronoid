use std::{net::TcpStream, ops::DerefMut, str::FromStr};

use bevy::{camera::visibility::Visibility, ui::widget::Text};
use bevy_ecs::{
    query::With,
    resource::Resource,
    system::{Commands, Query, Res, ResMut},
};
use bevy_state::state::NextState;
use dronoid::protocol::{AuthenticationRequest, ClientMessage};
use tungstenite::{Bytes, stream::MaybeTlsStream};

use crate::{ConnectPage, InfoLabel, PlayerName, ServerHost, ServerPort, SpawnPoint, State};

#[derive(Resource)]
pub struct Connection(pub tungstenite::WebSocket<MaybeTlsStream<TcpStream>>);

impl Connection {
    pub fn new(websocket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        Self(websocket)
    }
}

pub fn connect(
    mut connect_page: Query<&mut Visibility, With<ConnectPage>>,
    mut info_label: Query<&mut Text, With<InfoLabel>>,
    // mut r_state: ResMut<State>,
    mut state: ResMut<NextState<State>>,
    server_host: ResMut<ServerHost>,
    server_port: ResMut<ServerPort>,
    mut commands: Commands,
) {
    let mut connect_page_visibility = connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Hidden;
    let addr = format!("ws://{}:{}", server_host.0, server_port.0).to_string();
    let mut info_label = info_label.iter_mut().next().unwrap();
    info_label.0 = format!("Connecting to {}...", addr).to_string();
    if let Ok(uri) = tungstenite::http::Uri::from_str(addr.as_str()) {
        match tungstenite::connect(tungstenite::ClientRequestBuilder::new(uri)) {
            Ok((websocket, _)) => {
                commands.insert_resource(Connection::new(websocket));
                info_label.0 = "Sending auth request".to_string().to_string();
                // state.
                state.set(State::AuthenticateSendRequest);
                // *r_state.deref_mut() = State::AuthenticateSendRequest;
            }
            Err(err) => {
                info_label.0 = format!("Connection failed: {}", err).to_string();
                state.set(State::ShowConnectPage);
                // *r_state.deref_mut() = State::ShowConnectPage;
            }
        }
    } else {
        info_label.0 = "URI build failed".to_string().to_string();
        state.set(State::ShowConnectPage);
        // *r_state.deref_mut() = State::ShowConnectPage;
    }
}

pub fn authenticate_send_request(
    mut info_label: Query<&mut Text, With<InfoLabel>>,
    // mut r_state: ResMut<State>,
    mut state: ResMut<NextState<State>>,
    mut connection: ResMut<Connection>,
    player_name: Res<PlayerName>,
) {
    let mut info_label = info_label.iter_mut().next().unwrap();
    let maybe_auth_request = bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
        AuthenticationRequest {
            player_name: player_name.0.clone(),
        },
    ));
    if maybe_auth_request.is_err() {
        info_label.0 = format!(
            "Auth request serialization error: {}",
            maybe_auth_request.err().unwrap()
        );
        state.set(State::ShowConnectPage);
        // *r_state.deref_mut() = ;
        return;
    }
    let auth_request = maybe_auth_request.unwrap();
    let result = connection
        .0
        .write(tungstenite::Message::Binary(Bytes::from(auth_request)));

    if result.is_err() {
        info_label.0 = format!("Auth request send error: {}", result.err().unwrap());
        state.set(State::ShowConnectPage);
        // *r_state.deref_mut() = State::ShowConnectPage;
        return;
    }

    let result = connection.0.flush();
    if result.is_err() {
        info_label.0 = format!("Auth request send error: {}", result.err().unwrap());
        state.set(State::ShowConnectPage);
        // *r_state.deref_mut() = State::ShowConnectPage;
        return;
    }
    info_label.0 = "Waiting auth response".to_string().to_string();
    state.set(State::AuthenticateWaitResponse);
    // *r_state.deref_mut() = State::AuthenticateWaitResponse;
}

pub fn authenticate_wait_response(
    mut info_label: Query<&mut Text, With<InfoLabel>>,
    // mut r_state: ResMut<State>,
    mut state: ResMut<NextState<State>>,
    mut connection: ResMut<Connection>,
    mut spawn_point: ResMut<SpawnPoint>,
) {
    let mut info_label = info_label.iter_mut().next().unwrap();

    let maybe_response = connection.0.read();
    if maybe_response.is_err() {
        info_label.0 = format!(
            "Auth response read error: {}",
            maybe_response.err().unwrap()
        );
        state.set(State::ShowConnectPage);
        // *r_state.deref_mut() =;
        return;
    }

    let response = maybe_response.unwrap();
    if let tungstenite::Message::Binary(binary) = response {
        let maybe_auth_response = bson::deserialize_from_slice::<dronoid::protocol::ServerMessage>(
            binary.iter().as_slice(),
        );
        if maybe_auth_response.is_err() {
            info_label.0 = format!(
                "Auth response deserialization error: {}",
                maybe_auth_response.err().unwrap()
            );
            state.set(State::ShowConnectPage);
            // *r_state.deref_mut() = State::ShowConnectPage;
            return;
        }
        match maybe_auth_response.unwrap() {
            dronoid::protocol::ServerMessage::Response(
                dronoid::protocol::Response::AuthenticationResponse(auth_response),
            ) => {
                if !auth_response.result {
                    info_label.0 =
                        format!("Server declined authentication: {}", auth_response.text);
                    state.set(State::ShowConnectPage);
                    // *r_state.deref_mut() = State::ShowConnectPage;
                    return;
                }
                spawn_point.0 = auth_response.spawn_point;
                info_label.0 = format!("Authenticated: {}", auth_response.text).to_string();
                state.set(State::PrepareGame);
                // *r_state.deref_mut() = State::PrepareGame;
            }
            _ => {
                std::process::abort();
            }
        }
    } else {
        info_label.0 = "Unexpected non binary auth response".to_string();
        state.set(State::ShowConnectPage);
        // *r_state.deref_mut() = State::ShowConnectPage;
    }
}
