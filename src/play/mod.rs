pub mod building;
pub mod drone;
pub mod program;

use rapier2d::dynamics::RigidBodySet;

use crate::game::Order;

pub(crate) trait Play {
    fn discover_radius(&self) -> f32;
    fn position(&self, rigid_body_set: &RigidBodySet) -> (f32, f32);
    fn id(&self) -> i64;
    fn cycle(&mut self, delta: f32) -> Option<Vec<Order>>;
}
