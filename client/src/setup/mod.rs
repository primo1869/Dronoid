use bevy::{asset::AssetServer, camera::Camera2d, transform::components::Transform};
use bevy_ecs::system::{Commands, Res, ResMut};

use crate::GameSprites;

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
pub mod platform;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod platform;

pub fn setup_sprites(asset_server: Res<AssetServer>, mut game_sprites: ResMut<GameSprites>) {
    game_sprites.0.insert(
        dronoid::protocol::Kind::Mineral,
        (1. / 128., asset_server.load("textures/mineral.png")),
    );
    game_sprites.0.insert(
        dronoid::protocol::Kind::Dronoid,
        (3. / 128., asset_server.load("textures/dronoid.png")),
    );
    game_sprites.0.insert(
        dronoid::protocol::Kind::Factory,
        (6. / 128., asset_server.load("textures/factory.png")),
    );
    game_sprites.0.insert(
        dronoid::protocol::Kind::Spawn,
        (9. / 128., asset_server.load("textures/spawn.png")),
    );
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0., 0., 0.)));
}
