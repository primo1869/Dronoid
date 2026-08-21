use bevy_app::{App, Update};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    resource::Resource,
    schedule::Schedule,
    system::{Commands, Query, ResMut},
    world::World,
};
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
    geometry::{ColliderBuilder, ColliderSet, DefaultBroadPhase, NarrowPhase},
    math::{Vec2, Vector},
    pipeline::PhysicsPipeline,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    player::Player,
    protocol::{AuthenticationResponse, PlayerAction, ServerMessage},
    utils::is_name_valid,
};

const TICK_DURATION: f32 = 0.1;

pub(crate) enum Order {
    AddDrone,
}

pub(crate) struct RegisteredPlayer {
    id: i64,
    spawn_point: (f32, f32),
}

#[derive(Component)]
struct ZoneExtension {
    radius: f32,
}

#[derive(Component)]
struct RigidBody {
    hdl: RigidBodyHandle,
}

#[derive(Component)]
struct ProgramCounter {
    idx: u16,
}

#[derive(Component)]
struct Factory {
    auto_spawn: bool,
    cooldown: f32,
}

#[derive(Resource, Default)]
struct RapierBodies {
    set: RigidBodySet,
}

#[derive(Resource, Default)]
struct RapierColliders {
    set: ColliderSet,
}

#[derive(Resource, Default)]
struct RapierPipeline {
    physics_pipeline: PhysicsPipeline,
}

#[derive(Resource, Default)]
struct Rapier {
    gravity: Vector,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
}

fn process_factory(
    query: Query<&mut Factory>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
    mut commands: Commands,
) {
    for factory in query {
        if factory.cooldown < 0. && factory.cooldown - TICK_DURATION <= 0. {
            let factory_rigid_body = RigidBodyBuilder::fixed()
                .translation(Vec2::new(0., 0.))
                .build();
            let hdl = rapier_bodies.set.insert(factory_rigid_body);
            let factory_collider = ColliderBuilder::ball(5.).build();
            rapier_colliders
                .set
                .insert_with_parent(factory_collider, hdl, &mut rapier_bodies.set);
            commands.spawn((ZoneExtension { radius: 25. }, RigidBody { hdl }));
        }
    }
}

fn rapier_step(
    mut rapier: ResMut<Rapier>,
    mut rapier_pipeline: ResMut<RapierPipeline>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
) {
    rapier_pipeline.physics_pipeline.step(
        rapier.gravity,
        &rapier.integration_parameters,
        &mut rapier.island_manager,
        &mut rapier.broad_phase,
        &mut rapier.narrow_phase,
        &mut rapier_bodies.set,
        &mut rapier_colliders.set,
        &mut rapier.impulse_joint_set,
        &mut rapier.multibody_joint_set,
        &mut rapier.ccd_solver,
        &rapier.physics_hooks,
        &rapier.event_handler,
    );
}

pub(crate) async fn main_loop(players: Arc<Mutex<Vec<Player>>>) {
    let start_time = tokio::time::Instant::now();
    let mut time_mark = start_time.clone();

    let mut registered_players = HashMap::<String, RegisteredPlayer>::new();

    log::info!("Main loop is running...");

    App::new()
        .add_systems(Update, process_factory)
        .add_systems(Update, rapier_step)
        .insert_resource(Rapier::default())
        .insert_resource(RapierPipeline::default())
        .insert_resource(RapierBodies::default())
        .insert_resource(RapierColliders::default())
        .run();

    loop {
        cycle(
            TICK_DURATION,
            players.lock().await.as_mut(),
            // &mut drones,
            // &mut buildings,
            &mut rigid_body_set,
            &mut collider_set,
            &mut registered_players,
        )
        .await;

        // schedule.run(&mut world);
        let late_of = (tokio::time::Instant::now() - time_mark).as_secs_f32() / TICK_DURATION;
        if late_of > 1. {
            log::warn!("Server is late of {} tick(s)", late_of.trunc())
        }
        time_mark += tokio::time::Duration::from_secs_f32(late_of.ceil());

        tokio::time::sleep_until(time_mark).await;
    }
}

pub(crate) async fn cycle(
    delta: f32,
    players: &mut Vec<Player>,
    // drones: &mut MultiMap<i64, Drone>,
    // buildings: &mut MultiMap<i64, Box<dyn Play + Send>>,
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    registered_players: &mut HashMap<String, RegisteredPlayer>,
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
                        player.name = player_name.clone();
                        let maybe_registered_player = registered_players.get(&player_name);
                        if maybe_registered_player.is_some() {
                            let registered_player = maybe_registered_player.unwrap();
                            player.spawn_point = registered_player.spawn_point;
                            player.id = registered_player.id;
                        } else {
                            player.spawn_point = (
                                rand::random_range(-100f32..100f32),
                                rand::random_range(-100f32..100f32),
                            );
                            player.id = rand::random_range(i64::MIN..i64::MAX);
                            let rigid_body = RigidBodyBuilder::fixed()
                                .translation(Vector::new(
                                    player.spawn_point.0,
                                    player.spawn_point.1,
                                ))
                                .build();
                            let collider = ColliderBuilder::ball(1.).build();
                            let rigid_body_hdl = rigid_body_set.insert(rigid_body);
                            collider_set.insert_with_parent(
                                collider,
                                rigid_body_hdl,
                                rigid_body_set,
                            );
                            // buildings.insert(player.id, Box::new(Beacon::new(rigid_body_hdl)));
                            registered_players.insert(
                                player.name.clone(),
                                RegisteredPlayer {
                                    id: player.id,
                                    spawn_point: player.spawn_point,
                                },
                            );
                        }

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
                PlayerAction::PlaceFactory((pos_x, pos_y)) => {
                    // let player_buildings = buildings.get_vec(&player.id).unwrap();
                    // for building in player_buildings {
                    //     let building_pos = building.position(rigid_body_set);
                    //     if (pos_x - building_pos.0).powf(2.) + (pos_y - building_pos.1).powf(2.)
                    //         < building.discover_radius().powf(2.)
                    //     {
                    //         let rigid_body = RigidBodyBuilder::fixed()
                    //             .translation(Vector::new(pos_x, pos_y))
                    //             .build();
                    //         let collider = ColliderBuilder::ball(1.).build();
                    //         let rigid_body_hdl = rigid_body_set.insert(rigid_body);
                    //         collider_set.insert_with_parent(
                    //             collider,
                    //             rigid_body_hdl,
                    //             rigid_body_set,
                    //         );
                    //         buildings.insert(player.id, Box::new(Factory::new(rigid_body_hdl)));
                    //         break;
                    //     }
                    // }
                }
                PlayerAction::ControlFactory { id, control } => {
                    // let player_buildings = buildings.get_vec_mut(&player.id).unwrap();
                    // for building in player_buildings {
                    //     if building.id() == id {
                    //         match control {
                    //             FactoryControl::ManualSpawn => {}
                    //             FactoryControl::SetAutoSpawn(value) => {}
                    //         }
                    //         continue;
                    //     }
                    // }
                    // error
                }
            },
        }
        i += 1;
    }

    for idx in idxs_to_remove.iter().rev() {
        players.remove(*idx);
    }

    // for (_player_id, buildings) in buildings.iter_all_mut() {
    //     for building in buildings {
    //         if let Some(orders) = building.cycle(delta) {
    //             for order in orders {
    //                 match order {
    //                     Order::AddDrone => {
    //                         let building_pos = building.position(rigid_body_set);
    //                         let rigid_body = RigidBodyBuilder::fixed()
    //                             .translation(Vector::new(building_pos.0, building_pos.1))
    //                             .build();
    //                         let collider = ColliderBuilder::ball(1.).build();
    //                         let rigid_body_hdl = rigid_body_set.insert(rigid_body);
    //                         collider_set.insert_with_parent(
    //                             collider,
    //                             rigid_body_hdl,
    //                             rigid_body_set,
    //                         );
    //                         drones.insert(building.id(), Drone::new(building.id(), rigid_body_hdl));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}
