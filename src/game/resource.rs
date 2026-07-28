use bevy_ecs::resource::Resource;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use noise::NoiseFn;
use noise::Perlin;
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodySet,
    },
    geometry::{ColliderSet, DefaultBroadPhase, NarrowPhase},
    pipeline::PhysicsPipeline,
};
use std::collections::HashMap;
use tokio::time::Instant;

use crate::player::EnteringPlayer;
use crate::player::OnlinePlayer;

#[derive(Resource, Default)]
pub(crate) struct RapierIntegrationParameters(pub(crate) IntegrationParameters);

#[derive(Resource, Default)]
pub(crate) struct RapierIslandManager(pub(crate) IslandManager);

#[derive(Resource, Default)]
pub(crate) struct RapierBroadPhase(pub(crate) DefaultBroadPhase);

#[derive(Resource, Default)]
pub(crate) struct RapierNarrowPhase(pub(crate) NarrowPhase);

#[derive(Resource, Default)]
pub(crate) struct RapierImpulseJointSet(pub(crate) ImpulseJointSet);

#[derive(Resource, Default)]
pub(crate) struct RapierMultibodyJointSet(pub(crate) MultibodyJointSet);

#[derive(Resource, Default)]
pub(crate) struct RapierCCDSolver(pub(crate) CCDSolver);

#[derive(Resource, Default)]
pub(crate) struct RapierBodies(pub(crate) RigidBodySet);

#[derive(Resource, Default)]
pub(crate) struct RapierColliders(pub(crate) ColliderSet);

#[derive(Resource, Default)]
pub(crate) struct RapierPipeline(pub(crate) PhysicsPipeline);

#[derive(Resource)]
pub(crate) struct TransportStopper(pub(crate) Sender<()>);

#[derive(Resource)]
pub(crate) struct NewPlayerReceiver(pub(crate) Receiver<EnteringPlayer>);

#[derive(Resource, Default)]
pub(crate) struct Players(pub(crate) HashMap<u32, OnlinePlayer>);

#[derive(Resource)]
pub(crate) struct Controls(pub(crate) crate::Controls);

#[derive(Resource)]
pub(crate) struct Rules(pub(crate) crate::Rules);

#[derive(Resource)]
pub(crate) struct TerrainGenerator {
    noise_generator: Perlin,
    created: HashMap<(i32, i32), ()>,
}

impl TerrainGenerator {
    pub(crate) fn new(rules: &crate::Rules) -> Self {
        Self {
            noise_generator: Perlin::new(rules.terrain_seed as u32),
            created: HashMap::default(),
        }
    }

    pub(crate) fn put_mineral(&mut self, x: f32, y: f32, rules: &crate::Rules) -> bool {
        if !self.created.contains_key(&(x as i32, y as i32)) {
            let noise_at_point = self.noise_generator.get([
                (x * rules.terrain_scale) as f64,
                (y * rules.terrain_scale) as f64,
            ]);
            if noise_at_point > rules.mineral_threshold as f64
                && noise_at_point < (rules.mineral_threshold + 0.2) as f64
            {
                self.created.insert((x as i32, y as i32), ());
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Resource)]
pub(crate) struct Timers {
    pub(crate) last_cycle: Instant,
    #[cfg(debug_assertions)]
    pub(crate) last_info: Instant,
}

impl Default for Timers {
    fn default() -> Self {
        let now = Instant::now();
        Timers {
            last_cycle: now,
            #[cfg(debug_assertions)]
            last_info: now,
        }
    }
}
