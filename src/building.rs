// use rapier2d::dynamics::RigidBodyHandle;

// use crate::program::Program;

// pub(crate) struct Beacon {
//     rigid_body_hdl: RigidBodyHandle,
// }
// impl Beacon {
//     pub(crate) fn new(rigid_body_hdl: RigidBodyHandle) -> Self {
//         Self { rigid_body_hdl }
//     }
// }

// impl Play for Beacon {
//     fn discover_radius(&self) -> f32 {
//         25.
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

// pub(crate) struct Factory {
//     pub(crate) id: i64,
//     auto_spawn: bool,
//     pub(crate) program: Program,
//     rigid_body_hdl: RigidBodyHandle,
//     cooldown: f32,
// }

// impl Factory {
//     pub(crate) fn new(rigid_body_hdl: RigidBodyHandle) -> Self {
//         Self {
//             id: rand::random_range(i64::MIN..i64::MAX),
//             auto_spawn: false,
//             program: Program::default(),
//             rigid_body_hdl,
//             cooldown: 0.,
//         }
//     }
// }

// impl Play for Factory {
//     fn discover_radius(&self) -> f32 {
//         50.
//     }
//     fn position(&self, rigid_body_set: &RigidBodySet) -> (f32, f32) {
//         let position = rigid_body_set.get(self.rigid_body_hdl).unwrap().position();
//         (position.translation.x, position.translation.y)
//     }
//     fn id(&self) -> i64 {
//         self.id
//     }
//     fn cycle(&mut self, delta: f32) -> Option<Vec<Order>> {
//         if self.cooldown == 0. {
//             None
//         } else if self.cooldown - delta <= 0. {
//             Some(vec![Order::AddDrone])
//         } else {
//             self.cooldown -= delta;
//             None
//         }
//     }
// }
