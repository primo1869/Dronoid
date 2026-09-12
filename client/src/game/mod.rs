use bevy::{
    camera::{Camera2d, Projection, visibility::Visibility},
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    transform::components::Transform,
};
use bevy_ecs::{
    query::With,
    system::{Query, Res, ResMut},
};
use bevy_state::state::NextState;
use std::ops::DerefMut;

use crate::{ConnectPage, SpawnPoint, State};

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
pub mod platform;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod platform;

pub fn prepare_game(
    mut q_camera: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    mut q_connect_page: Query<&mut Visibility, With<ConnectPage>>,
    mut state: ResMut<NextState<State>>,
    r_spawn_point: Res<SpawnPoint>,
) {
    let mut connect_page_visibility = q_connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Hidden;

    let (mut projection, mut transform) = q_camera.iter_mut().next().unwrap();
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = 0.1;
    }

    *transform = Transform::from_xyz(r_spawn_point.0.0, r_spawn_point.0.1, 0.);
    state.set(State::ShowGame);
}

pub fn handle_camera(
    mut q_camera: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
    r_mouse_motion: Res<AccumulatedMouseMotion>,
    r_mouse_scroll: Res<AccumulatedMouseScroll>,
    r_mouse_button: Res<ButtonInput<MouseButton>>,
    r_keyboard_button: Res<ButtonInput<KeyCode>>,
    r_spawn_point: Res<SpawnPoint>,
) {
    let (mut projection, mut transform) = q_camera.iter_mut().next().unwrap();

    if let Projection::Orthographic(ortho) = projection.as_mut() {
        if r_mouse_button.pressed(MouseButton::Middle) {
            let factor = 10.;
            transform.translation.x += r_mouse_motion.delta.x / factor;
            transform.translation.y -= r_mouse_motion.delta.y / factor;
        } else {
            match r_mouse_scroll.unit {
                bevy::input::mouse::MouseScrollUnit::Pixel => {
                    ortho.scale -= r_mouse_scroll.delta.y;
                    ortho.scale = ortho.scale.clamp(0., 80.);
                }
                bevy::input::mouse::MouseScrollUnit::Line => {
                    ortho.scale -= r_mouse_scroll.delta.y / 100.;
                    ortho.scale = ortho.scale.clamp(0.05, 0.8);
                }
            }
        }

        if r_keyboard_button.just_pressed(KeyCode::Space) {
            transform.translation.x = r_spawn_point.0.0;
            transform.translation.y = r_spawn_point.0.1;
        }
    }
}
