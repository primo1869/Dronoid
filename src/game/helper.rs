use bevy_ecs::system::Query;
use rapier2d::dynamics::RigidBodySet;

use crate::{game::component, protocol};

#[cfg(debug_assertions)]
pub(crate) fn count_kind(entities: &Vec<&protocol::Kind>, kind: &protocol::Kind) -> usize {
    entities
        .iter()
        .fold(0, |x, y| if *y == kind { x + 1 } else { x })
}

pub(crate) fn get_entities_in_zone(
    player_id: u32,
    all_entities: Vec<(&component::RapierObject, &component::Kind, &component::Id)>,
    rigid_body_set: &RigidBodySet,
    zone_extenders: Query<(
        &component::ZoneExtension,
        &component::RapierObject,
        &component::Owned,
    )>,
) -> Vec<protocol::Entity> {
    let mut entities_in_zone = Vec::new();
    for (rapier_position, kind, id) in all_entities {
        let position = rapier_position.position(rigid_body_set);
        if is_in_player_zone(
            position.0,
            position.1,
            &player_id,
            zone_extenders.iter().collect(),
            rigid_body_set,
        ) {
            entities_in_zone.push(protocol::Entity {
                pos: (position.0, position.1),
                kind: kind.0.clone(),
                id: id.0.clone(),
            });
        }
    }
    entities_in_zone
}

pub(crate) fn is_in_player_zone(
    pos_x: f32,
    pos_y: f32,
    id: &u32,
    zone_extenders: Vec<(
        &component::ZoneExtension,
        &component::RapierObject,
        &component::Owned,
    )>,
    rigid_body_set: &RigidBodySet,
) -> bool {
    for (zone_extension, rapier_position, owner_id) in zone_extenders {
        if owner_id.0 != *id {
            continue;
        }
        let translation = rigid_body_set
            .get(rapier_position.rapier_hdl)
            .unwrap()
            .position()
            .translation;
        if (pos_x - translation.x).powf(2.) + (pos_y - translation.y).powf(2.)
            > zone_extension.radius.powf(2.)
        {
            continue;
        }
        return true;
    }
    return false;
}

pub(crate) fn gen_id() -> u32 {
    rand::random_range(u32::MIN..u32::MAX)
}
