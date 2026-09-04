use bevy::{
    input_focus::{FocusCause, InputFocus},
    prelude::*,
};
use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Query, ResMut},
};

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

pub fn connect_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                // input_focus.set(entity, FocusCause::Pressed);
                // **text = "Press".to_string();
                // *color = PRESSED_BUTTON.into();
                // *border_color = BorderColor::all(RED);

                // button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity, FocusCause::Pressed);
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

pub fn zoom_camera(mut query: Query<&mut Projection, With<Camera2d>>) {
    for mut projection in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 0.1;
        }
    }
}
