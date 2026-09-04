use bevy::{log::LogPlugin, prelude::*};
use bevy_app::{App, PluginGroup, Update};
use bevy_ecs::schedule::{IntoScheduleConfigs, common_conditions::resource_equals};
use clap::Parser;

use crate::ecs::{resources, systems};

pub mod ecs;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = "127.0.0.1".to_string())]
    addr: String,
    #[arg(default_value_t = dronoid::defaults::PORT)]
    port: u16,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dronoid::init_logger();

    App::new()
        // -------------------- PLUGINS
        .add_plugins((
            DefaultPlugins.build().disable::<LogPlugin>(),
            bevy_framepace::FramepacePlugin,
        ))
        // -------------------- RESOURCES
        .insert_resource(resources::Entities::default())
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .insert_resource(resources::GameSprites::default())
        .insert_resource(resources::ServerHost { 0: args.addr })
        .insert_resource(resources::ServerPort { 0: args.port })
        .insert_resource(resources::SpawnPoint::default())
        .insert_resource(resources::State::default())
        .insert_resource(resources::CurrentDisplayResolution::default())
        // -------------------- SYSTEMS
        .add_systems(PreStartup, systems::setup::setup_display)
        .add_systems(
            Update,
            systems::setup::setup_connect_page.run_if(resource_equals::<resources::State>(
                resources::State::SetupConnectPage,
            )),
        )
        .add_systems(
            Update,
            (systems::show_connect_page)
                .run_if(resource_equals::<resources::State>(
                    resources::State::ShowConnectPage,
                ))
                .run_if(resource_equals::<resources::State>(
                    resources::State::Connect,
                )),
        )
        .add_systems(
            Update,
            (systems::connect).run_if(resource_equals::<resources::State>(
                resources::State::Connect,
            )),
        )
        .add_systems(
            Update,
            (systems::zoom_camera).run_if(resource_equals::<resources::State>(
                resources::State::ShowGame,
            )),
        )
        // -------------------- RUN
        .run();
    anyhow::Ok(())
}
