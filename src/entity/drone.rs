use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use uuid::Uuid;

use crate::entity::place::Place;

pub(crate) struct Drone {
    pub(crate) factory_id: Uuid,
    pub(crate) direction: (f32, f32),
    pub(crate) throttle: bool,
    pub(crate) program_counter: u16,
    pub(crate) rigid_body_hdl: RigidBodyHandle,
}

impl Drone {
    fn new(factory_id: Uuid, rigid_body_hdl: RigidBodyHandle) -> Self {
        Self {
            factory_id,
            direction: (1., 0.),
            program_counter: 1,
            throttle: false,
            rigid_body_hdl,
        }
    }
}

impl Place for Drone {
    fn discover_radius(&self) -> f32 {
        10.
    }
    fn position(&self, rigid_body_set: RigidBodySet) -> (f32, f32) {
        let position = rigid_body_set.get(self.rigid_body_hdl).unwrap().position();
        (position.translation.x, position.translation.y)
    }
}
