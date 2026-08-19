use rapier2d::dynamics::RigidBodySet;
use uuid::Uuid;

pub(crate) trait Play {
    fn discover_radius(&self) -> f32;
    fn position(&self, rigid_body_set: RigidBodySet) -> (f32, f32);
    fn id(&self) -> Uuid;
}
