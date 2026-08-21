// use rapier2d::dynamics::RigidBodyHandle;

// pub(crate) struct Drone {
//     pub(crate) factory_id: i64,
//     pub(crate) direction: (f32, f32),
//     pub(crate) throttle: bool,
//     pub(crate) program_counter: u16,
//     pub(crate) rigid_body_hdl: RigidBodyHandle,
// }

// impl Drone {
//     pub(crate) fn new(factory_id: i64, rigid_body_hdl: RigidBodyHandle) -> Self {
//         Self {
//             factory_id,
//             direction: (1., 0.),
//             program_counter: 1,
//             throttle: false,
//             rigid_body_hdl,
//         }
//     }
// }

// impl Play for Drone {
//     fn discover_radius(&self) -> f32 {
//         10.
//     }
//     fn position(&self, rigid_body_set: &RigidBodySet) -> (f32, f32) {
//         let position = rigid_body_set.get(self.rigid_body_hdl).unwrap().position();
//         (position.translation.x, position.translation.y)
//     }
//     fn id(&self) -> i64 {
//         i64::default()
//     }
//     fn cycle(&mut self, _delta: f32) -> Option<Vec<Order>> {
//         None
//     }
// }
