pub fn show_game(
    mut q_entities: Query<&mut Transform>,

    r_connection: ResMut<resources::web::Connection>,
    mut r_entities: ResMut<resources::Entities>,
    r_sprites: Res<resources::GameSprites>,
    mut commands: Commands,
) {
    use crossbeam_channel::TryRecvError::{Disconnected, Empty};
    use std::process::abort;

    match r_connection.message_receiver.try_recv() {
        Err(Disconnected) => {
            abort();
        }
        Err(Empty) => {
            return;
        }
        Ok(message) => match message {
            ServerMessage::State(state) => {
                for entity_state in state.entities_in_zone {
                    if let Some(existing_entity) = r_entities.0.get(&entity_state.id) {
                        let mut transform = q_entities.get_mut(*existing_entity).unwrap();
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
            _ => {
                abort();
            }
        },
    }
}
