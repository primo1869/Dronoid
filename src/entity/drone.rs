use uuid::Uuid;

use crate::entity::discover::Discover;

pub(crate) struct Drone {
    pub(crate) factory_id: Uuid,
    pub(crate) direction: (f32, f32),
    pub(crate) throttle: bool,
    pub(crate) program_counter: u16,
}

impl Drone {
    fn new(factory_id: Uuid) -> Self {
        Self {
            factory_id,
            direction: (1., 0.),
            program_counter: 1,
            throttle: false,
        }
    }
}

impl Discover for Drone {
    fn radius(&self) -> f32 {
        10.
    }
}
