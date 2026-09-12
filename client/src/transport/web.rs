use bevy_ecs::{
    message::MessageWriter,
    resource::Resource,
    system::{Commands, Res, ResMut},
};
use bevy_state::state::NextState;
use dronoid::protocol::AuthenticationRequest;
use wasm_bindgen::JsCast;
use web_sys::js_sys::Uint8Array;

use crate::app::{InfoMessage, PlayerName, ProgramOptions, ServerMessage, SpawnPoint, State};

#[derive(Resource)]
pub struct Connection {
    pub websocket: web_sys::WebSocket,
    pub message_receiver: crossbeam_channel::Receiver<dronoid::protocol::ServerMessage>,
}

impl Connection {
    pub fn new(websocket: web_sys::WebSocket) -> Self {
        use wasm_bindgen::prelude::Closure;
        use web_sys::MessageEvent;

        let (tx, rx) = crossbeam_channel::unbounded();
        let on_message = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            use web_sys::js_sys;

            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let vec = Uint8Array::new(&abuf);
                let server_message = bson::deserialize_from_slice::<dronoid::protocol::ServerMessage>(
                    vec.to_vec().as_slice(),
                );
                tx.send(server_message.unwrap()).unwrap();
            }
        });
        websocket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        Self {
            websocket,
            message_receiver: rx,
        }
    }
}

pub fn connect(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<State>>,
    program_options: Res<ProgramOptions>,
    mut commands: Commands,
) {
    let hostname = program_options.hostname.clone();
    let mut protocol = "ws";
    if let true = program_options.tls {
        protocol = "wss";
    }
    let addr = format!(
        "{}://{}:{}/{}",
        protocol, hostname, program_options.port, program_options.suffix
    )
    .to_string();
    info_label.write(InfoMessage(
        format!("Connecting to {}...", addr).to_string(),
    ));

    match web_sys::WebSocket::new(addr.as_str()) {
        Ok(websocket) => {
            websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);
            commands.insert_resource(Connection::new(websocket));
            info_label.write(InfoMessage(format!("Sending auth request").to_string()));
            state.set(State::AuthenticateSendRequest);
        }
        Err(err) => {
            info_label.write(InfoMessage(
                format!("Connection failed: {:?}", err).to_string(),
            ));
            state.set(State::ShowConnectPage);
        }
    }
}

pub fn authenticate_send_request(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<State>>,
    r_connection: ResMut<Connection>,
    r_player_name: Res<PlayerName>,
) {
    use dronoid::protocol::ClientMessage;
    use web_sys::WebSocket;

    // let mut info_label = q_info_label.iter_mut().next().unwrap();
    if r_connection.websocket.ready_state() != WebSocket::OPEN {
        return;
    }
    let maybe_auth_request = bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
        AuthenticationRequest {
            player_name: r_player_name.0.clone(),
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
    match r_connection
        .websocket
        .send_with_u8_array(maybe_auth_request.unwrap().as_slice())
    {
        Err(err) => {
            info_label.write(InfoMessage(format!("Error: {:?}", err)));
            state.set(State::ShowConnectPage);
        }
        _ => {
            info_label.write(InfoMessage(format!("Waiting for authentication response")));
            state.set(State::AuthenticateWaitResponse);
        }
    }
}

pub fn authenticate_wait_response(
    mut info_label: MessageWriter<InfoMessage>,
    mut state: ResMut<NextState<State>>,
    r_connection: ResMut<Connection>,
    mut r_spawn_point: ResMut<SpawnPoint>,
) {
    use crossbeam_channel::TryRecvError::{Disconnected, Empty};
    use std::process::abort;

    // let mut info_label = q_info_label.iter_mut().next().unwrap();
    match r_connection.message_receiver.try_recv() {
        Err(Disconnected) => {
            abort();
        }
        Err(Empty) => {
            return;
        }
        Ok(message) => match message {
            dronoid::protocol::ServerMessage::Response(
                dronoid::protocol::Response::AuthenticationResponse(auth_response),
            ) => {
                if !auth_response.result {
                    info_label.write(InfoMessage(format!(
                        "Authentication declined: {}",
                        auth_response.text
                    )));
                    state.set(State::ShowConnectPage);
                } else {
                    r_spawn_point.0 = auth_response.spawn_point;
                    info_label.write(InfoMessage("".to_string()));
                    state.set(State::PrepareGame);
                }
            }
            _ => {
                abort();
            }
        },
    }
}

pub fn read_server_messages(
    connection: Res<Connection>,
    mut server_messages: MessageWriter<ServerMessage>,
) {
    use crossbeam_channel::TryRecvError::{Disconnected, Empty};
    use std::process::abort;

    match connection.message_receiver.try_recv() {
        Err(Disconnected) => {
            abort();
        }
        Err(Empty) => {
            return;
        }
        Ok(message) => {
            server_messages.write(ServerMessage(message));
        }
    }
}
