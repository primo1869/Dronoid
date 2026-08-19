use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use uuid::Uuid;

use crate::entity::{place::Place, program::Program};

// pub(crate) enum Building {
//     SpawnBeacon(SpawnBeacon),
//     Factory(Factory),
// }

pub(crate) struct SpawnBeacon {
    rigid_body_hdl: RigidBodyHandle,
}

impl Place for SpawnBeacon {
    fn discover_radius(&self) -> f32 {
        25.
    }
    fn position(&self, rigid_body_set: RigidBodySet) -> (f32, f32) {
        let position = rigid_body_set.get(self.rigid_body_hdl).unwrap().position();
        (position.translation.x, position.translation.y)
    }
}

pub(crate) struct Factory {
    pub(crate) id: Uuid,
    auto_spawn: bool,
    pub(crate) program: Program,
    rigid_body_hdl: RigidBodyHandle,
}

impl Factory {
    fn new(rigid_body_hdl: RigidBodyHandle) -> Self {
        Self {
            id: Uuid::new_v4(),
            auto_spawn: false,
            program: Program::default(),
            rigid_body_hdl,
        }
    }
}

impl Place for Factory {
    fn discover_radius(&self) -> f32 {
        50.
    }
    fn position(&self, rigid_body_set: RigidBodySet) -> (f32, f32) {
        let position = rigid_body_set.get(self.rigid_body_hdl).unwrap().position();
        (position.translation.x, position.translation.y)
    }
}
