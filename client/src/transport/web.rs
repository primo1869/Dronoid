
#[derive(Resource)]
pub struct Connection {
    pub websocket: web_sys::WebSocket,
    pub message_receiver: crossbeam_channel::Receiver<ServerMessage>,
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
                let server_message =
                    bson::deserialize_from_slice::<ServerMessage>(vec.to_vec().as_slice());
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
    mut q_connect_page: Query<&mut Visibility, With<components::ConnectPage>>,
    mut q_info_label: Query<&mut Text, With<components::InfoLabel>>,
    mut r_state: ResMut<resources::State>,
    mut commands: Commands,
) {
    let mut connect_page_visibility = q_connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Hidden;
    let hostname = ;
    let addr = format!("wss://{}/dronoid/ws", hostname).to_string();
    let mut info_label = q_info_label.iter_mut().next().unwrap();
    info_label.0 = format!("Connecting to {}...", addr).to_string();

    match web_sys::WebSocket::new(addr.as_str()) {
        Ok(websocket) => {
            websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);
            commands.insert_resource(resources::web::Connection::new(websocket));
            info_label.0 = format!("Sending auth request").to_string();
            *r_state.deref_mut() = resources::State::AuthenticateSendRequest;
        }
        Err(err) => {
            info_label.0 = format!("Connection failed: {:?}", err).to_string();
            *r_state.deref_mut() = resources::State::ShowConnectPage;
        }
    }
}

pub fn authenticate_send_request(
    mut q_info_label: Query<&mut Text, With<components::InfoLabel>>,
    mut r_state: ResMut<resources::State>,
    r_connection: ResMut<resources::web::Connection>,
    r_player_name: Res<resources::PlayerName>,
) {
    use dronoid::protocol::ClientMessage;
    use web_sys::WebSocket;

    let mut info_label = q_info_label.iter_mut().next().unwrap();
    if r_connection.websocket.ready_state() != WebSocket::OPEN {
        return;
    }
    let maybe_auth_request = bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
        AuthenticationRequest {
            player_name: r_player_name.0.clone(),
        },
    ));
    if maybe_auth_request.is_err() {
        info_label.0 = format!(
            "Auth request serialization error: {}",
            maybe_auth_request.err().unwrap()
        );
        *r_state.deref_mut() = resources::State::ShowConnectPage;
        return;
    }
    match r_connection
        .websocket
        .send_with_u8_array(maybe_auth_request.unwrap().as_slice())
    {
        Err(err) => {
            info_label.0 = format!("Error: {:?}", err);
        }
        _ => {
            info_label.0 = format!("Waiting for authentication response");
            *r_state.deref_mut() = resources::State::AuthenticateWaitResponse;
        }
    }
}

pub fn authenticate_wait_response(
    mut q_info_label: Query<&mut Text, With<components::InfoLabel>>,
    mut r_state: ResMut<resources::State>,
    r_connection: ResMut<resources::web::Connection>,
    mut r_spawn_point: ResMut<resources::SpawnPoint>,
) {
    use crossbeam_channel::TryRecvError::{Disconnected, Empty};
    use std::process::abort;

    let mut info_label = q_info_label.iter_mut().next().unwrap();
    match r_connection.message_receiver.try_recv() {
        Err(Disconnected) => {
            abort();
        }
        Err(Empty) => {
            return;
        }
        Ok(message) => match message {
            ServerMessage::Response(dronoid::protocol::Response::AuthenticationResponse(
                auth_response,
            )) => {
                if !auth_response.result {
                    info_label.0 = format!("Authentication declined: {}", auth_response.text);
                    *r_state.deref_mut() = resources::State::ShowConnectPage;
                } else {
                    r_spawn_point.0 = auth_response.spawn_point;
                    info_label.0 = "".to_string();
                    *r_state.deref_mut() = resources::State::PrepareGame;
                }
            }
            _ => {
                abort();
            }
        },
    }
}