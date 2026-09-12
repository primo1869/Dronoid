#![forbid(unsafe_code)]

use crate::{
    game::{handle_camera, platform::show_game, prepare_game},
    setup::{platform::setup_display, setup_camera, setup_sprites},
    transport::platform::{
        Connection, authenticate_send_request, authenticate_wait_response, connect,
    },
    ui::{
        platform::{handle_connect_page, setup_connect_page},
        show_connect_page,
    },
};
use bevy::prelude::*;
use bevy::{
    asset::AssetMetaCheck, input_focus::tab_navigation::TabNavigationPlugin, log::LogPlugin,
};
use bevy_app::{App, PluginGroup, Update};
use bevy_ecs::schedule::IntoScheduleConfigs;
use rand::random_range;
use std::collections::HashMap;

mod game;
mod setup;
mod transport;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub struct HostField;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub struct PortField;

#[derive(Component)]
pub struct PlayerNameField;

#[derive(Component)]
pub struct ConnectButton;

#[derive(Component)]
pub struct InfoLabel;

#[derive(Component)]
pub struct ConnectPage;

#[derive(Message)]
pub struct ServerMessage(dronoid::protocol::ServerMessage);

#[derive(Resource, Default)]
pub struct SpawnPoint(pub (f32, f32));

#[derive(Resource, Default)]
pub struct Entities(pub HashMap<u32, Entity>);

#[derive(Resource, Default)]
pub struct GameSprites(pub HashMap<dronoid::protocol::Kind, (f32, Handle<Image>)>);

#[derive(Resource)]
pub struct ServerHost(pub String);

#[derive(Resource)]
pub struct ServerPort(pub u16);

#[derive(Resource, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    ShowConnectPage,
    HandleConnectPage,
    Connect,
    AuthenticateSendRequest,
    AuthenticateWaitResponse,
    PrepareGame,
    ShowGame,
}

impl Default for State {
    fn default() -> Self {
        Self::ShowConnectPage
    }
}

#[derive(Resource)]
pub struct PlayerName(pub String);

// #[cfg(not(target_arch = "wasm32"))]
// #[derive(Resource, Default)]
// pub struct CurrentDisplayResolution(pub (u32, u32));

fn gen_name() -> String {
    format!("Player{}", random_range(u8::MIN..u8::MAX)).to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn init() -> (String, String, u16) {
    (gen_name(), "127.0.0.1".to_string(), 8080)
}

#[cfg(target_arch = "wasm32")]
fn init() -> (String, String, String) {
    (
        gen_name(),
        web_sys::window().unwrap().location().hostname().unwrap(),
        web_sys::window().unwrap().location().port().unwrap(),
    )
}

fn main() -> anyhow::Result<()> {
    let (player_name, hostname, port) = init();
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            TabNavigationPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .insert_resource(Entities::default())
        .insert_resource(GameSprites::default())
        .insert_resource(ServerHost(hostname))
        .insert_resource(ServerPort(port))
        .insert_resource(PlayerName(player_name))
        .insert_resource(SpawnPoint::default())
        .init_state::<State>()
        .add_message::<ServerMessage>()
        // .insert_resource(CurrentDisplayResolution::default())
        .add_systems(PreStartup, setup_display)
        .add_systems(Startup, setup_sprites)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_connect_page)
        .add_systems(
            Update,
            show_connect_page.run_if(in_state(State::ShowConnectPage)),
        )
        .add_systems(
            Update,
            handle_connect_page.run_if(in_state(State::HandleConnectPage)),
        )
        .add_systems(Update, connect.run_if(in_state(State::Connect)))
        .add_systems(
            Update,
            authenticate_send_request.run_if(
                in_state(State::AuthenticateSendRequest).and_then(resource_exists::<Connection>),
            ),
        )
        .add_systems(
            Update,
            authenticate_wait_response.run_if(
                in_state(State::AuthenticateWaitResponse).and_then(resource_exists::<Connection>),
            ),
        )
        .add_systems(Update, prepare_game.run_if(in_state(State::PrepareGame)))
        .add_systems(Update, show_game.run_if(in_state(State::ShowGame)))
        .add_systems(Update, handle_camera.run_if(in_state(State::ShowGame)))
        .run();
    anyhow::Ok(())
}
