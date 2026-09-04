use bevy::{log::LogPlugin, prelude::*};
use bevy_app::{App, PluginGroup, Startup, Update};
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
        .add_plugins((
            DefaultPlugins.build().disable::<LogPlugin>(),
            bevy_framepace::FramepacePlugin,
        ))
        .insert_resource(resources::Entities::default())
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .insert_resource(resources::GameSprites::default())
        .insert_resource(resources::ServerHost { 0: args.addr })
        .insert_resource(resources::ServerPort { 0: args.port })
        .insert_resource(resources::SpawnPoint::default())
        .insert_resource(resources::State::default())
        .insert_resource(resources::CurrentDisplayResolution::default())
        .add_systems(PreStartup, systems::setup::setup_display)
        .add_systems(Startup, systems::setup::setup_connect_page)
        .add_systems(
            Update,
            (systems::connect_system).run_if(resource_equals::<resources::State>(
                resources::State { playing: false },
            )),
        )
        .add_systems(
            Update,
            (systems::zoom_camera).run_if(resource_equals::<resources::State>(resources::State {
                playing: true,
            })),
        )
        .run();
    anyhow::Ok(())
}
