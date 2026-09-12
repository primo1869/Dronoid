use bevy_ecs::{
    message::MessageWriter,
    resource::Resource,
    system::{Commands, Res, ResMut},
};
use bevy_state::state::NextState;
use dronoid::protocol::{AuthenticationRequest, ClientMessage};
use std::{net::TcpStream, str::FromStr};
use tungstenite::{Bytes, Message, stream::MaybeTlsStream};

use crate::app::{InfoMessage, PlayerName, ProgramOptions, ServerMessage, SpawnPoint, State};

#[derive(Resource)]
pub struct Connection(pub tungstenite::WebSocket<MaybeTlsStream<TcpStream>>);

impl Connection {
    pub fn new(websocket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        Self(websocket)
    }
}

pub fn connect(
    mut info_label: MessageWriter<InfoMessage>,
    mut next_state: ResMut<NextState<State>>,
    program_options: Res<ProgramOptions>,
    mut commands: Commands,
) {
    let mut protocol = "ws";
    if let true = program_options.tls {
        protocol = "wss";
    }
    let addr = format!(
        "{}://{}:{}/{}",
        protocol, program_options.hostname, program_options.port, program_options.suffix
    )
    .to_string();
    info_label.write(InfoMessage(addr.clone()));
    if let Ok(uri) = tungstenite::http::Uri::from_str(addr.as_str()) {
        match tungstenite::connect(tungstenite::ClientRequestBuilder::new(uri)) {
            Ok((websocket, _)) => {
                commands.insert_resource(Connection::new(websocket));
                info_label.write(InfoMessage("Sending auth request".to_string()));
                next_state.set(State::AuthenticateSendRequest);
            }
            Err(err) => {
                info_label.write(InfoMessage(
                    format!("Connection failed: {}", err).to_string(),
                ));
                next_state.set(State::HandleConnectPage);
            }
        }
    } else {
        info_label.write(InfoMessage("URI build failed".to_string()));
        next_state.set(State::HandleConnectPage);
    }
}

pub fn authenticate_send_request(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<State>>,
    mut connection: ResMut<Connection>,
    player_name: Res<PlayerName>,
) {
    let maybe_auth_request = bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
        AuthenticationRequest {
            player_name: player_name.0.clone(),
        },
    ));
    if maybe_auth_request.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth request serialization error: {}",
            maybe_auth_request.err().unwrap()
        )));
        state.set(State::ShowConnectPage);
        return;
    }
    let auth_request = maybe_auth_request.unwrap();
    let result = connection
        .0
        .write(tungstenite::Message::Binary(Bytes::from(auth_request)));

    if result.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth request send error: {}",
            result.err().unwrap()
        )));
        state.set(State::ShowConnectPage);
        return;
    }

    let result = connection.0.flush();
    if result.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth request send error: {}",
            result.err().unwrap()
        )));
        state.set(State::ShowConnectPage);
        return;
    }
    info_label.write(InfoMessage("Waiting auth response".to_string().to_string()));
    state.set(State::AuthenticateWaitResponse);
}

pub fn authenticate_wait_response(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<State>>,
    mut connection: ResMut<Connection>,
    mut spawn_point: ResMut<SpawnPoint>,
) {
    let maybe_response = connection.0.read();
    if maybe_response.is_err() {
        info_label.write(InfoMessage(format!(
            "Auth response read error: {}",
            maybe_response.err().unwrap()
        )));
        state.set(State::ShowConnectPage);
        return;
    }

    let response = maybe_response.unwrap();
    if let tungstenite::Message::Binary(binary) = response {
        let maybe_auth_response = bson::deserialize_from_slice::<dronoid::protocol::ServerMessage>(
            binary.iter().as_slice(),
        );
        if maybe_auth_response.is_err() {
            info_label.write(InfoMessage(format!(
                "Auth response deserialization error: {}",
                maybe_auth_response.err().unwrap()
            )));
            state.set(State::ShowConnectPage);
            return;
        }
        match maybe_auth_response.unwrap() {
            dronoid::protocol::ServerMessage::Response(
                dronoid::protocol::Response::AuthenticationResponse(auth_response),
            ) => {
                if !auth_response.result {
                    info_label.write(InfoMessage(format!(
                        "Server declined authentication: {}",
                        auth_response.text
                    )));
                    state.set(State::ShowConnectPage);
                    return;
                }
                spawn_point.0 = auth_response.spawn_point;
                info_label.write(InfoMessage(
                    format!("Authenticated: {}", auth_response.text).to_string(),
                ));
                state.set(State::PrepareGame);
            }
            _ => {
                std::process::abort();
            }
        }
    } else {
        info_label.write(InfoMessage(
            "Unexpected non binary auth response".to_string(),
        ));
        state.set(State::ShowConnectPage);
    }
}

pub fn read_server_messages(
    mut connection: ResMut<Connection>,
    mut server_messages: MessageWriter<ServerMessage>,
) {
    while connection.0.can_read() {
        if let Ok(Message::Binary(message)) = connection.0.read() {
            let server_message = bson::deserialize_from_slice::<dronoid::protocol::ServerMessage>(
                &message.iter().as_slice(),
            )
            .unwrap();

            server_messages.write(ServerMessage(server_message));
        }
    }
}
