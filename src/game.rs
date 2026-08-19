use std::str::FromStr;

use crate::{
    player::Player,
    protocol::{AuthenticationResponse, PlayerAction, ServerMessage},
    utils::is_name_valid,
};

pub(crate) async fn cycle(players: &mut Vec<Player>) {
    let mut idxs_to_remove = Vec::<usize>::new();
    let mut i = 0;
    let player_names: Vec<String> = players.iter().map(|player| player.name.clone()).collect();
    for player in &mut *players {
        let maybe_player_action = player.receiver.try_recv();
        match maybe_player_action {
            Err(err) => match err {
                tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                    idxs_to_remove.push(i);
                }
                tokio::sync::mpsc::error::TryRecvError::Empty => continue,
            },
            Ok(player_action) => match player_action {
                PlayerAction::Authentication { player_name } => {
                    let response: (bool, ServerMessage) = if player.authenticated {
                        log::trace!("Already authenticated");
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("Already authenticated").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    } else if !is_name_valid(&player_name) {
                        log::trace!("Invalid name {}", player_name);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("Invalid name").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    } else if player_names
                        .iter()
                        .find(|&other_name| other_name == &player_name)
                        .is_some()
                    {
                        log::trace!("Name already taken: {}", player_name);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("A player already has this name").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    } else {
                        log::trace!("Authenticated: {}", player_name);
                        player.authenticated = true;
                        player.name = player_name;
                        (
                            true,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: true,
                                text: String::from_str("Welcome").unwrap(),
                                pos_x: 0.,
                                pos_y: 0.,
                                pos_z: 0.,
                            }),
                        )
                    };
                    if player.sender.send(response).await.is_err() {
                        idxs_to_remove.push(i);
                    }
                }
                PlayerAction::PlaceFactory {
                    #[allow(unused)]
                    pos_x,
                    #[allow(unused)]
                    pos_y,
                    #[allow(unused)]
                    pos_z,
                } => {
                    continue;
                }
            },
        }
        i += 1;
    }

    for idx in idxs_to_remove.iter().rev() {
        players.remove(*idx);
    }
}
