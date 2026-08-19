use rapier2d::dynamics::RigidBodySet;

pub(crate) trait Place {
    fn discover_radius(&self) -> f32;
    fn position(&self, rigid_body_set: RigidBodySet) -> (f32, f32);
}
