use bevy_ecs::{component::Component, system::Commands};
use rapier2d::{
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
    geometry::{ColliderBuilder, ColliderSet},
    math::Vec2,
};

#[derive(Component)]
pub(crate) struct Kind(pub(crate) crate::protocol::Kind);

#[derive(Component)]
pub(crate) struct ZoneExtension {
    pub(crate) radius: f32,
}

#[derive(Component)]
pub(crate) struct HasExtended;

#[derive(Component)]
pub(crate) struct RapierObject {
    pub(crate) rapier_hdl: RigidBodyHandle,
}

impl RapierObject {
    pub fn position(&self, rigid_bodies: &RigidBodySet) -> (f32, f32) {
        let rigid_body = rigid_bodies.get(self.rapier_hdl).unwrap();
        (
            rigid_body.position().translation.x,
            rigid_body.position().translation.y,
        )
    }
}

#[derive(Component)]
pub(crate) struct Owned(pub(crate) u32);

#[derive(Component)]
pub(crate) struct Id(pub(crate) u32);

// impl Id {
//     pub fn new() -> Self {
//         Self { 0: () }
//     }
// }

#[derive(Component, Default)]
pub(crate) struct Factory {
    pub(crate) auto_spawn: bool,
    pub(crate) must_spawn: bool,
    pub(crate) cooldown: f32,
}

impl Factory {
    pub(crate) fn spawn_dronoid(
        &mut self,
        mut rapier_bodies: &mut RigidBodySet,
        rapier_colliders: &mut ColliderSet,
        commands: &mut Commands,
    ) {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Vec2::new(0., 0.))
            .build();
        let rapier_hdl = rapier_bodies.insert(rigid_body);
        let collider = ColliderBuilder::ball(3.).build();
        rapier_colliders.insert_with_parent(collider, rapier_hdl, &mut rapier_bodies);
        commands.spawn((
            ZoneExtension { radius: 25. },
            RapierObject { rapier_hdl },
            Kind {
                0: crate::protocol::Kind::Dronoid,
            },
        ));
    }
}

#[derive(Component, Default)]
pub(crate) struct Resource;
