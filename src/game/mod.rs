pub mod component;
pub mod helper;
pub mod resource;
pub mod system;

use bevy::prelude::*;
use bevy_app::{App, AppExit, Startup};
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use tracing::info;

use crate::Controls;
use crate::Error;
use crate::Rules;
use crate::player::EnteringPlayer;

pub fn run(
    rules: Rules,
    controls: Controls,
    transport_stopper_tx: Sender<()>,
    new_player_rx: Receiver<EnteringPlayer>,
) -> super::Result<()> {
    let exit = App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, system::startup)
        .add_systems(FixedFirst, system::physics)
        .add_systems(
            FixedUpdate,
            (
                #[cfg(debug_assertions)]
                system::debug,
                system::cycle,
                system::new_players,
                system::terrain,
                system::actions,
                system::factories,
                system::sync,
                system::flush,
            )
                .chain(),
        )
        .insert_resource(Time::<Fixed>::from_seconds(rules.tick_duration as f64))
        .insert_resource(resource::RapierBroadPhase::default())
        .insert_resource(resource::RapierCCDSolver::default())
        .insert_resource(resource::RapierImpulseJointSet::default())
        .insert_resource(resource::RapierIntegrationParameters::default())
        .insert_resource(resource::RapierIslandManager::default())
        .insert_resource(resource::RapierMultibodyJointSet::default())
        .insert_resource(resource::RapierNarrowPhase::default())
        .insert_resource(resource::RapierPipeline::default())
        .insert_resource(resource::RapierBodies::default())
        .insert_resource(resource::RapierColliders::default())
        .insert_resource(resource::Controls { 0: controls })
        .insert_resource(resource::TransportStopper {
            0: transport_stopper_tx,
        })
        .insert_resource(resource::NewPlayerReceiver { 0: new_player_rx })
        .insert_resource(resource::Players::default())
        .insert_resource(resource::TerrainGenerator::new(&rules))
        .insert_resource(resource::Rules { 0: rules })
        .insert_resource(resource::Timers::default())
        .run();

    info!(sender = "Exit", "Syncing and cleaning");
    match exit {
        AppExit::Error(err_num) => {
            let num = err_num.get();
            match num {
                1 => Err(Error::StopError(num)),
                _ => {
                    todo!();
                }
            }
        }

        AppExit::Success => Ok(()),
    }
}
