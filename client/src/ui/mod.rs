use std::ops::DerefMut;

use bevy::camera::visibility::Visibility;
use bevy_color::Color;
use bevy_ecs::{query::With, system::Query};

use crate::ConnectPage;

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
pub mod platform;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod platform;

pub fn show_connect_page(mut connect_page: Query<&mut Visibility, With<ConnectPage>>) {
    let mut connect_page_visibility = connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Visible;
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
