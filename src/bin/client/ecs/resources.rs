use bevy::{asset::Handle, image::Image};
use bevy_ecs::{entity::Entity, resource::Resource};
use dronoid::protocol::Kind;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct SpawnPoint(pub (f32, f32));

#[derive(Resource, Default)]
pub struct Entities(pub HashMap<u32, Entity>);

#[derive(Resource, Default)]
pub struct GameSprites(pub HashMap<Kind, (f32, Handle<Image>)>);

#[derive(Resource, Default, PartialEq)]
pub struct State {
    pub playing: bool,
}

#[derive(Resource)]
pub struct PlayerCredentials {
    pub name: String,
}

#[derive(Resource)]
pub struct ServerHost(pub String);

#[derive(Resource)]
pub struct ServerPort(pub u16);

#[derive(Resource, Default)]
pub struct CurrentDisplayResolution(pub (u32, u32));
