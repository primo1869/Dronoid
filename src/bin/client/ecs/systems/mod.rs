use bevy::{
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    text::TextSection,
};
use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Query, ResMut},
};
use std::{ops::DerefMut, str::FromStr};
use tokio_tungstenite::tungstenite;

use crate::ecs::{components::ConnectButton, resources};

pub mod setup;

// fn read_server_message(
//     mut entities_query: Query<&mut Transform, With<Sprite>>,
//     entities: ResMut<Entities>,
//     game_sprites: Res<GameSprites>,
//     receiver: ResMut<ServerMessageReceiver>,
//     mut commands: Commands,
// ) {
//     loop {
//         let maybe_message = receiver.0.try_recv();
//         match maybe_message {
//             Err(crossbeam_channel::TryRecvError::Empty) => return,
//             Ok(message) => match message {
//                 ServerMessage::State(state) => {
//                     for entity_info in state.entities_in_zone {
//                         let maybe_existing_entity = entities.0.get(&entity_info.id);
//                         if maybe_existing_entity.is_some() {
//                             let existing_entity = maybe_existing_entity.unwrap();
//                             let mut transform = entities_query.get_mut(*existing_entity).unwrap();
//                             transform.translation.x = entity_info.pos.0;
//                             transform.translation.y = entity_info.pos.1;
//                             continue;
//                         }
//                         let sprite_info = game_sprites.0.get(&entity_info.kind).unwrap();
//                         let mut transform =
//                             Transform::from_xyz(entity_info.pos.0, entity_info.pos.1, 0.);
//                         transform.scale = Vec3::new(sprite_info.0, sprite_info.0, sprite_info.0);
//                         commands.spawn((Sprite::from_image(sprite_info.1.clone()), transform));
//                     }
//                 }
//                 _ => {}
//             },
//             _ => {
//                 abort();
//             }
//         }
//     }
// }

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn show_connect_page(
    mut input_focus: ResMut<InputFocus>,
    mut server_host: ResMut<resources::ServerHost>,
    mut server_port: ResMut<resources::ServerPort>,
    mut interaction_query: Query<
        (
            Entity,
            &ConnectButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, _, interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                server_host.0 = text.get_text().to_string();
                input_focus.set(entity, FocusCause::Pressed);
            }
            Interaction::Hovered => {
                input_focus.set(entity, FocusCause::Pressed);
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                input_focus.clear();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn connect(
    mut state: ResMut<resources::State>,
    server_host: ResMut<resources::ServerHost>,
    server_port: ResMut<resources::ServerPort>,
    mut commands: Commands,
) {
    let result = if let Ok(uri) = tungstenite::http::Uri::from_str(
        format!("ws://{}:{}", server_host.0, server_port.0).as_str(),
    ) {
        if let Ok((websocket, _)) =
            tungstenite::connect(tungstenite::ClientRequestBuilder::new(uri))
        {
            commands.insert_resource(resources::Connection::new(websocket));
            true
        } else {
            false
        }
    } else {
        false
    };
    if result {
        *state.deref_mut() = resources::State::ShowGame;
    } else {
        *state.deref_mut() = resources::State::SetupConnectPage;
    }
}

pub fn zoom_camera(mut query: Query<&mut Projection, With<Camera2d>>) {
    for mut projection in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 0.1;
        }
    }
}
