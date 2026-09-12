use std::ops::DerefMut;

use bevy::{camera::visibility::Visibility, ui::widget::Text};
use bevy_color::Color;
use bevy_ecs::{message::MessageReader, query::With, system::Query};

use crate::app::{ConnectPage, InfoLabel, InfoMessage};

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
pub mod platform;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod platform;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

pub fn show_connect_page(mut connect_page: Query<&mut Visibility, With<ConnectPage>>) {
    let mut connect_page_visibility = connect_page.iter_mut().next().unwrap();
    *connect_page_visibility.deref_mut() = Visibility::Visible;
}

pub fn info_label(
    mut info_events: MessageReader<InfoMessage>,
    mut info_label: Query<&mut Text, With<InfoLabel>>,
) {
    let mut info_label = info_label.iter_mut().next().unwrap();

    for info_event in info_events.read() {
        info_label.0 = info_event.0.clone();
    }
}
