use uuid::Uuid;

use crate::entity::{discover::Discover, program::Program};

pub(crate) enum Building {
    SpawnBeacon(SpawnBeacon),
    Factory(Factory),
}

pub(crate) struct SpawnBeacon;

impl Discover for SpawnBeacon {
    fn radius(&self) -> f32 {
        25.
    }
}

pub(crate) struct Factory {
    pub(crate) id: Uuid,
    auto_spawn: bool,
    pub(crate) program: Program,
}

impl Factory {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            auto_spawn: false,
            program: Program::default(),
        }
    }
}

impl Discover for Factory {
    fn radius(&self) -> f32 {
        50.
    }
}
