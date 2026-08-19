use std::{str::FromStr, sync::Arc};

use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodySet,
    },
    geometry::{ColliderSet, DefaultBroadPhase, NarrowPhase},
    math::Vector,
    pipeline::PhysicsPipeline,
};
use tokio::sync::Mutex;

use crate::{
    entity::{building::Building, drone::Drone},
    player::Player,
    protocol::{AuthenticationResponse, PlayerAction, ServerMessage},
    utils::is_name_valid,
};

const TICK_DURATION: f32 = 0.1;

pub(crate) async fn main_loop(players: Arc<Mutex<Vec<Player>>>) {
    let start_time = tokio::time::Instant::now();
    let mut time_mark = start_time.clone();
    let mut drones = Vec::<Drone>::new();
    let mut buildings = Vec::<Building>::new();

    let gravity = Vector::new(0.0, 0.0);
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    log::info!("Main loop is running...");
    loop {
        cycle(
            players.lock().await.as_mut(),
            &mut drones,
            &mut buildings,
            &mut rigid_body_set,
            &mut collider_set,
        )
        .await;
        physics_pipeline.step(
            gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            &physics_hooks,
            &event_handler,
        );
        let late_of = (tokio::time::Instant::now() - time_mark).as_secs_f32() / TICK_DURATION;
        if late_of > 1. {
            log::warn!("Server is late of {} tick(s)", late_of.trunc())
        }
        time_mark += tokio::time::Duration::from_secs_f32(late_of.ceil());

        tokio::time::sleep_until(time_mark).await;
    }
}

pub(crate) async fn cycle(
    players: &mut Vec<Player>,
    drones: &mut Vec<Drone>,
    buildings: &mut Vec<Building>,
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
) {
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
                        log::trace!("{}: Already authenticated", player.addr);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("Already authenticated").unwrap(),
                                spawn_point: (0., 0.),
                            }),
                        )
                    } else if !is_name_valid(&player_name) {
                        log::trace!("{}: Invalid name {}", player.addr, player_name);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("Invalid name").unwrap(),
                                spawn_point: (0., 0.),
                            }),
                        )
                    } else if player_names
                        .iter()
                        .find(|&other_name| other_name == &player_name)
                        .is_some()
                    {
                        log::trace!("{}: Name already taken: {}", player.addr, player_name);
                        idxs_to_remove.push(i);
                        (
                            false,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: false,
                                text: String::from_str("A player already has this name").unwrap(),
                                spawn_point: (0., 0.),
                            }),
                        )
                    } else {
                        log::trace!("{}: Authenticated: {}", player.addr, player_name);
                        player.authenticated = true;
                        player.name = player_name;
                        player.spawn_point = (
                            rand::random_range(-100f32..100f32),
                            rand::random_range(-100f32..100f32),
                        );
                        (
                            true,
                            ServerMessage::AuthenticationResponse(AuthenticationResponse {
                                result: true,
                                text: String::from_str("Welcome").unwrap(),
                                spawn_point: player.spawn_point,
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

    for building in &mut *buildings {
        match building {
            Building::Factory(factory) => {
                for drone in &mut *drones {
                    if drone.factory_id == factory.id {
                        let _ = factory.program.exec(drone);
                    }
                }
            }
            _ => {}
        }
    }
}
