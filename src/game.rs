use bevy::MinimalPlugins;
use bevy_app::{App, PluginGroup, ScheduleRunnerPlugin, Update};
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{Commands, Query, Res, ResMut},
};
use bevy_time::Time;
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
    geometry::{ColliderBuilder, ColliderSet, DefaultBroadPhase, NarrowPhase},
    math::{Vec2, Vector},
    pipeline::PhysicsPipeline,
};
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};
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
struct RapierGravity(Vector);

#[derive(Resource, Default)]
struct RapierIntegrationParameters(IntegrationParameters);

#[derive(Resource, Default)]
struct RapierIslandManager(IslandManager);

#[derive(Resource, Default)]
struct RapierBroadPhase(DefaultBroadPhase);

#[derive(Resource, Default)]
struct RapierNarrowPhase(NarrowPhase);

#[derive(Resource, Default)]
struct RapierImpulseJointSet(ImpulseJointSet);

#[derive(Resource, Default)]
struct RapierMultibodyJointSet(MultibodyJointSet);

#[derive(Resource, Default)]
struct RapierCCDSolver(CCDSolver);

#[derive(Resource, Default)]
struct RapierPhysicsHook(());

#[derive(Resource, Default)]
struct RapierEventHandler(());

#[derive(Resource, Default)]
struct RapierBodies(RigidBodySet);

#[derive(Resource, Default)]
struct RapierColliders(ColliderSet);

#[derive(Resource, Default)]
struct RapierPipeline(PhysicsPipeline);

#[derive(Resource, Default)]
struct Players {
    online: Vec<Player>,
    registered: HashMap<String, RegisteredPlayer>,
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
            let hdl = rapier_bodies.0.insert(factory_rigid_body);
            let factory_collider = ColliderBuilder::ball(5.).build();
            rapier_colliders
                .0
                .insert_with_parent(factory_collider, hdl, &mut rapier_bodies.0);
            commands.spawn((ZoneExtension { radius: 25. }, RigidBody { hdl }));
        }
    }
}

fn rapier_step(
    mut rapier_gravity: ResMut<RapierGravity>,
    mut rapier_integration_parameters: ResMut<RapierIntegrationParameters>,
    mut rapier_island_manager: ResMut<RapierIslandManager>,
    mut rapier_broad_phase: ResMut<RapierBroadPhase>,
    mut rapier_narrow_phase: ResMut<RapierNarrowPhase>,
    mut rapier_impulse_joint_set: ResMut<RapierImpulseJointSet>,
    mut rapier_multibody_joint_set: ResMut<RapierMultibodyJointSet>,
    mut rapier_ccd_solver: ResMut<RapierCCDSolver>,
    mut rapier_physics_hook: ResMut<RapierPhysicsHook>,
    mut rapier_event_handler: ResMut<RapierEventHandler>,
    mut rapier_pipeline: ResMut<RapierPipeline>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
) {
    rapier_pipeline.0.step(
        rapier_gravity.0,
        &rapier_integration_parameters.0,
        &mut rapier_island_manager.0,
        &mut rapier_broad_phase.0,
        &mut rapier_narrow_phase.0,
        &mut rapier_bodies.0,
        &mut rapier_colliders.0,
        &mut rapier_impulse_joint_set.0,
        &mut rapier_multibody_joint_set.0,
        &mut rapier_ccd_solver.0,
        &rapier_physics_hook.0,
        &rapier_event_handler.0,
    );
}

pub(crate) async fn main_loop(players: Arc<Mutex<Vec<Player>>>) {
    let start_time = tokio::time::Instant::now();
    let mut time_mark = start_time.clone();

    let mut registered_players = HashMap::<String, RegisteredPlayer>::new();

    log::info!("Main loop is running...");
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_systems(Update, process_factory)
        .add_systems(Update, rapier_step)
        .add_systems(Update, cycle)
        .insert_resource(RapierGravity::default())
        .insert_resource(RapierBroadPhase::default())
        .insert_resource(RapierCCDSolver::default())
        .insert_resource(RapierEventHandler::default())
        .insert_resource(RapierImpulseJointSet::default())
        .insert_resource(RapierIntegrationParameters::default())
        .insert_resource(RapierIslandManager::default())
        .insert_resource(RapierMultibodyJointSet::default())
        .insert_resource(RapierNarrowPhase::default())
        .insert_resource(RapierPhysicsHook::default())
        .insert_resource(RapierPipeline::default())
        .insert_resource(RapierBodies::default())
        .insert_resource(RapierColliders::default())
        // .insert_resource(FixedTime::new_from_secs(FIXED_TIMESTEP))
        .run();
}

fn cycle(
    time: Res<'_, Time>,
    rigid_body_set: ResMut<'_, RapierBodies>,
    collider_set: ResMut<'_, RapierColliders>,
    players: ResMut<'_, Players>,
) {
    let mut idxs_to_remove = Vec::<usize>::new();
    let mut i = 0;
    let player_names: Vec<String> = players
        .online
        .iter()
        .map(|player| player.name.clone())
        .collect();
    for player in &mut *players.online {
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
                        let maybe_registered_player = players.registered.get(&player_name);
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
                            let rigid_body_hdl = rigid_body_set.0.insert(rigid_body);
                            collider_set.0.insert_with_parent(
                                collider,
                                rigid_body_hdl,
                                &mut rigid_body_set.0,
                            );
                            // buildings.insert(player.id, Box::new(Beacon::new(rigid_body_hdl)));
                            players.registered.insert(
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
                    if player.sender.send(response).unwrap().is_err() {
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
        players.online.remove(*idx);
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
