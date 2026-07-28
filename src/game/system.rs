use bevy::prelude::*;
use bevy_app::AppExit;
use bevy_ecs::system::{Commands, Query, ResMut};
use crossbeam_channel::TryRecvError;
use rapier2d::{
    dynamics::RigidBodyBuilder,
    geometry::ColliderBuilder,
    math::{Vec2, Vector},
};
use std::num::NonZero;
use tokio::time::Instant;
use tracing::{info, warn};

#[cfg(debug_assertions)]
use std::time::Duration;
#[cfg(debug_assertions)]
use tracing::debug;

use crate::game::resource;
use crate::game::{component::HasExtended, helper};
use crate::player::OnlinePlayer;
use crate::{game::component, protocol};

pub(crate) fn startup(rules: Res<resource::Rules>) {
    let rules_str = format!("{:#?}", rules.0);
    info!(sender = "Game", "{rules_str}");
    info!(sender = "Game", "Waiting for players");
}

#[cfg(debug_assertions)]
pub(crate) fn debug(entities: Query<&component::Kind>, mut timers: ResMut<resource::Timers>) {
    let now = Instant::now();
    let kinds = entities.iter().map(|x| &x.0).collect();
    if now - timers.last_info > Duration::from_secs_f32(10.) {
        debug!(
            sender = "Debug",
            "Stat: Minerals: {}, Factories: {}, Dronoids: {},  SpawnBeacon: {}",
            helper::count_kind(&kinds, &protocol::Kind::Mineral),
            helper::count_kind(&kinds, &protocol::Kind::Factory),
            helper::count_kind(&kinds, &protocol::Kind::Dronoid),
            helper::count_kind(&kinds, &protocol::Kind::Spawn),
        );
        timers.last_info = now;
    }
}

pub(crate) fn cycle(
    controls: ResMut<resource::Controls>,
    transport_stopper: Res<resource::TransportStopper>,
    rules: Res<resource::Rules>,
    mut timers: ResMut<resource::Timers>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = Instant::now();
    let delta = (now - timers.last_cycle).as_secs_f32();
    timers.last_cycle = now;
    if delta > rules.0.tick_duration * 2. {
        let late = (delta / rules.0.tick_duration) as u32;
        warn!(
            sender = "Game",
            "Delta is {delta} seconds. Server is late of {late} ticks"
        );
    }

    if controls.0.stopped() {
        let app_exit = if transport_stopper.0.send(()).is_err() {
            AppExit::Error(NonZero::new(1).unwrap())
        } else {
            AppExit::Success
        };
        exit.write(app_exit);
    }
}

pub(crate) fn new_players(
    new_player_rx: ResMut<resource::NewPlayerReceiver>,
    mut players: ResMut<resource::Players>,
    mut rapier_bodies: ResMut<resource::RapierBodies>,
    mut rapier_colliders: ResMut<resource::RapierColliders>,
    rules: Res<resource::Rules>,
    mut commands: Commands,
) {
    while let Ok(new_player) = new_player_rx.0.try_recv() {
        let player_name = new_player.name.clone();
        info!(sender = "Game", "Player '{player_name}' joined");
        players.0.insert(
            new_player.id.clone(),
            OnlinePlayer::new(
                new_player.message_sender,
                new_player.action_receiver,
                new_player.name,
                rules.0.starting_minerals,
            ),
        );
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(rapier2d::math::Vector::new(
                new_player.spawn_point.0,
                new_player.spawn_point.1,
            ))
            .build();
        let rapier_hdl = rapier_bodies.0.insert(rigid_body);
        let collider = ColliderBuilder::cuboid(10., 10.).build();
        rapier_colliders
            .0
            .insert_with_parent(collider, rapier_hdl, &mut rapier_bodies.0);
        commands.spawn((
            component::Kind {
                0: protocol::Kind::Spawn,
            },
            component::Owned {
                0: new_player.id.clone(),
            },
            component::ZoneExtension { radius: 100. },
            component::RapierObject { rapier_hdl },
            component::Id {
                0: helper::gen_id(),
            },
        ));
    }
}

pub(crate) fn terrain(
    zone_extenders: Query<
        (Entity, &component::ZoneExtension, &component::RapierObject),
        Without<HasExtended>,
    >,
    mut terrain_generator: ResMut<resource::TerrainGenerator>,
    rules: Res<resource::Rules>,
    mut rapier_bodies: ResMut<resource::RapierBodies>,
    mut rapier_colliders: ResMut<resource::RapierColliders>,
    mut commands: Commands,
) {
    for (entity, zone_extender, rapier_object) in zone_extenders.iter() {
        let zone_position = rapier_object.position(&rapier_bodies.0);
        let radius = zone_extender.radius.ceil() as i32;
        for x in -radius..radius {
            for y in -radius..radius {
                let mineral_x = zone_position.0 + x as f32;
                let mineral_y = zone_position.1 + y as f32;
                if (mineral_x - zone_position.0).powf(2.) + (mineral_y - zone_position.1).powf(2.)
                    < radius.pow(2) as f32
                    && terrain_generator.put_mineral(mineral_x, mineral_y, &rules.0)
                {
                    let rigid_body = RigidBodyBuilder::fixed()
                        .translation(Vec2::new(mineral_x, mineral_y))
                        .build();
                    let rapier_hdl = rapier_bodies.0.insert(rigid_body);
                    let collider = ColliderBuilder::cuboid(1., 1.).build();
                    rapier_colliders.0.insert_with_parent(
                        collider,
                        rapier_hdl,
                        &mut rapier_bodies.0,
                    );
                    commands.spawn((
                        component::Resource::default(),
                        component::RapierObject { rapier_hdl },
                        component::Kind {
                            0: protocol::Kind::Mineral,
                        },
                        component::Id {
                            0: helper::gen_id(),
                        },
                    ));
                }
            }
        }
        commands.entity(entity).insert(HasExtended);
    }
}

pub(crate) fn actions(
    zone_extenders: Query<(
        &component::ZoneExtension,
        &component::RapierObject,
        &component::Owned,
    )>,
    mut factories: Query<&mut component::Factory>,
    mut rapier_bodies: ResMut<resource::RapierBodies>,
    mut rapier_colliders: ResMut<resource::RapierColliders>,
    mut players: ResMut<resource::Players>,
    rules: Res<resource::Rules>,
    mut commands: Commands,
) {
    let mut ids_to_remove = Vec::<u32>::new();

    for (id, player) in &mut players.0 {
        loop {
            match player.action_receiver.try_recv() {
                Err(err) => match err {
                    TryRecvError::Disconnected => {
                        ids_to_remove.push(id.clone());
                        break;
                    }
                    TryRecvError::Empty => break,
                },
                Ok(player_action) => match player_action {
                    protocol::Action::PlaceFactory((pos_x, pos_y)) => {
                        let may_build = helper::is_in_player_zone(
                            pos_x,
                            pos_y,
                            id,
                            zone_extenders.iter().collect(),
                            &rapier_bodies.0,
                        );

                        if may_build {
                            let rigid_body = RigidBodyBuilder::fixed()
                                .translation(Vec2::new(pos_x, pos_y))
                                .build();
                            let rapier_hdl = rapier_bodies.0.insert(rigid_body);
                            let collider = ColliderBuilder::cuboid(10., 10.).build();
                            rapier_colliders.0.insert_with_parent(
                                collider,
                                rapier_hdl,
                                &mut rapier_bodies.0,
                            );
                            let id = helper::gen_id();
                            player.owned_factories.insert(
                                id,
                                commands
                                    .spawn((
                                        component::ZoneExtension {
                                            radius: rules.0.zone_extensions.factory,
                                        },
                                        component::RapierObject { rapier_hdl },
                                        component::Factory::default(),
                                        component::Kind {
                                            0: protocol::Kind::Factory,
                                        },
                                        component::Id { 0: id },
                                    ))
                                    .id(),
                            );
                        }

                        player.messages.push(protocol::ServerMessage::Response(
                            protocol::Response::PlaceFactory { result: may_build },
                        ));
                    }
                    protocol::Action::ControlFactory(control) => {
                        let maybe_bevy_entity = player.owned_factories.get(&control.id);
                        if maybe_bevy_entity.is_none() {
                            player.to_kick = true;
                            break;
                        }
                        let bevy_entity = maybe_bevy_entity.unwrap();
                        let mut factory = factories.get_mut(*bevy_entity).unwrap();
                        match control.order {
                            protocol::FactoryOrder::ManualSpawn => {
                                factory.must_spawn = true;
                            }
                            protocol::FactoryOrder::SetAutoSpawn(value) => {
                                factory.auto_spawn = value;
                            }
                        }
                    }
                },
            }
        }
    }
}

pub(crate) fn factories(
    mut query: Query<&mut component::Factory>,
    mut rapier_bodies: ResMut<resource::RapierBodies>,
    mut rapier_colliders: ResMut<resource::RapierColliders>,
    rules: Res<resource::Rules>,
    mut commands: Commands,
) {
    for mut factory in &mut query {
        if factory.cooldown < 0. {
            if factory.must_spawn || factory.auto_spawn {
                factory.spawn_dronoid(&mut rapier_bodies.0, &mut rapier_colliders.0, &mut commands);
                factory.cooldown = 5.;
                if factory.must_spawn {
                    factory.must_spawn = false
                }
            }
        } else {
            factory.cooldown -= rules.0.tick_duration;
        }
    }
}

pub(crate) fn physics(
    rapier_integration_parameters: Res<resource::RapierIntegrationParameters>,
    mut rapier_island_manager: ResMut<resource::RapierIslandManager>,
    mut rapier_broad_phase: ResMut<resource::RapierBroadPhase>,
    mut rapier_narrow_phase: ResMut<resource::RapierNarrowPhase>,
    mut rapier_impulse_joint_set: ResMut<resource::RapierImpulseJointSet>,
    mut rapier_multibody_joint_set: ResMut<resource::RapierMultibodyJointSet>,
    mut rapier_ccd_solver: ResMut<resource::RapierCCDSolver>,
    mut rapier_pipeline: ResMut<resource::RapierPipeline>,
    mut rapier_bodies: ResMut<resource::RapierBodies>,
    mut rapier_colliders: ResMut<resource::RapierColliders>,
) {
    rapier_pipeline.0.step(
        Vector::new(0., 0.),
        &rapier_integration_parameters.0,
        &mut rapier_island_manager.0,
        &mut rapier_broad_phase.0,
        &mut rapier_narrow_phase.0,
        &mut rapier_bodies.0,
        &mut rapier_colliders.0,
        &mut rapier_impulse_joint_set.0,
        &mut rapier_multibody_joint_set.0,
        &mut rapier_ccd_solver.0,
        &mut (),
        &mut (),
    );
}

pub(crate) fn sync(
    entities: Query<(&component::RapierObject, &component::Kind, &component::Id)>,
    zone_extenders: Query<(
        &component::ZoneExtension,
        &component::RapierObject,
        &component::Owned,
    )>,
    rapier_bodies: Res<resource::RapierBodies>,
    mut players: ResMut<resource::Players>,
) {
    for (id, player) in &mut players.0 {
        let mut state = protocol::State::default();
        state.entities_in_zone = helper::get_entities_in_zone(
            id.clone(),
            entities.iter().collect(),
            &rapier_bodies.0,
            zone_extenders,
        );
        state.minerals_cnt = player.minerals_cnt;
        player.messages.push(protocol::ServerMessage::State(state));
    }
}

pub(crate) fn flush(mut players: ResMut<resource::Players>) {
    let mut ids_to_remove = Vec::<u32>::new();
    for (id, player) in &mut players.0 {
        if player.to_kick || player.flush_messages().is_err() {
            ids_to_remove.push(id.clone());
        }
    }
    for id in ids_to_remove {
        players.0.remove(&id);
    }
}
