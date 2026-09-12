use bevy::{
    camera::{Camera2d, Projection, visibility::Visibility},
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    sprite::Sprite,
    transform::components::Transform,
};
use bevy_ecs::{
    message::MessageReader,
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use bevy_state::state::NextState;
use std::ops::DerefMut;

use crate::app::{ConnectPage, Entities, GameSprites, ServerMessage, SpawnPoint, State};

pub fn prepare_game(
    mut camera: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    mut connect_page: Query<&mut Visibility, With<ConnectPage>>,
    mut state: ResMut<NextState<State>>,
    spawn_point: Res<SpawnPoint>,
) {
    let mut connect_page_visibility = connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Hidden;

    let (mut projection, mut transform) = camera.iter_mut().next().unwrap();
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = 0.1;
    }

    *transform = Transform::from_xyz(spawn_point.0.0, spawn_point.0.1, 0.);
    state.set(State::ShowGame);
}

pub fn handle_camera(
    mut camera: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard_button: Res<ButtonInput<KeyCode>>,
    spawn_point: Res<SpawnPoint>,
) {
    let (mut projection, mut transform) = camera.iter_mut().next().unwrap();

    if let Projection::Orthographic(ortho) = projection.as_mut() {
        if mouse_button.pressed(MouseButton::Right) {
            let factor = 10.;
            transform.translation.x += mouse_motion.delta.x / factor;
            transform.translation.y -= mouse_motion.delta.y / factor;
        } else {
            let zoom_speed = 0.2;
            match mouse_scroll.unit {
                bevy::input::mouse::MouseScrollUnit::Pixel => {
                    #[cfg(target_arch = "wasm32")]
                    let delta = mouse_scroll.delta.y / 100.0;
                    #[cfg(not(target_arch = "wasm32"))]
                    let delta = mouse_scroll.delta.y;

                    ortho.scale *= 1.0 - (delta * zoom_speed);
                }
                bevy::input::mouse::MouseScrollUnit::Line => {
                    ortho.scale *= 1.0 - (mouse_scroll.delta.y * zoom_speed * 0.5);
                }
            }
            ortho.scale = ortho.scale.clamp(0.05, 80.0);
        }

        if keyboard_button.just_pressed(KeyCode::Space) {
            transform.translation.x = spawn_point.0.0;
            transform.translation.y = spawn_point.0.1;
        }
    }
}

pub fn show_game(
    mut server_messages: MessageReader<ServerMessage>,
    mut entities: Query<&mut Transform>,
    // mut info_label: Query<&mut Text, With<InfoLabel>>,
    mut r_entities: ResMut<Entities>,
    r_sprites: Res<GameSprites>,
    mut commands: Commands,
) {
    for server_message in server_messages.read() {
        match &server_message.0 {
            dronoid::protocol::ServerMessage::Response(_response) => {}
            dronoid::protocol::ServerMessage::State(state) => {
                for entity_state in state.entities_in_zone.iter() {
                    if let Some(existing_entity) = r_entities.0.get(&entity_state.id) {
                        let mut transform = entities.get_mut(*existing_entity).unwrap();
                        transform.translation.x = entity_state.pos.0;
                        transform.translation.y = entity_state.pos.1;
                    } else {
                        let (size, image_hdl) = r_sprites.0.get(&entity_state.kind).unwrap();
                        let mut transform =
                            Transform::from_xyz(entity_state.pos.0, entity_state.pos.1, 0.);
                        transform.scale.x = *size;
                        transform.scale.y = *size;
                        let entity =
                            commands.spawn((transform, Sprite::from_image(image_hdl.clone())));
                        r_entities.0.insert(entity_state.id, entity.id());
                    }
                }
            }
        }
    }
}
