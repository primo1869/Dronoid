use bevy::{sprite::Sprite, transform::components::Transform};
use bevy_ecs::{
    message::MessageReader,
    system::{Commands, Query, Res, ResMut},
};

use crate::{Entities, GameSprites, ServerMessage};

pub fn show_game(
    mut server_messages: MessageReader<ServerMessage>,
    mut entities: Query<&mut Transform>,
    // mut info_label: Query<&mut Text, With<InfoLabel>>,
    mut r_entities: ResMut<Entities>,
    r_sprites: Res<GameSprites>,
    mut commands: Commands,
) {
    // let mut info_label = info_label.iter_mut().next().unwrap();

    // let mut cursor = server_messages.get_cursor();
    // for server_message in cursor.read(&server_messages) {
    for server_message in server_messages.read() {
        // if r_connection.0.can_read() {
        // let maybe_message = r_connection.0.read();
        // if maybe_message.is_err() {
        //     info_label.0 = format!("Read error: {}", maybe_message.err().unwrap());
        //     return;
        // }
        // let message = maybe_message.unwrap();
        // if let tungstenite::Message::Binary(bin) = message {
        // let maybe_server_message =
        //     bson::deserialize_from_slice::<ServerMessage>(bin.iter().as_slice());
        // if maybe_server_message.is_err() {
        //     info_label.0 = "Deserialization error".to_string();
        //     return;
        // }
        // let server_message = maybe_server_message.unwrap();
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
        // } else {
        //     info_label.0 = "Unexpected non binary message from server".to_string();
        // }
    }
}
