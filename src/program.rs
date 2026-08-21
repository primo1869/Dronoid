// use crate::program::_Instruction::{ChangeDirection, Jump, Throttle};

pub(crate) enum _Instruction {
    Jump(u16),
    Throttle(bool),
    ChangeDirection(f32, f32),
}

pub(crate) struct _Program {
    instructions: Vec<_Instruction>,
}

impl _Program {
    // pub(crate) fn exec(&self, drone: &mut Drone) -> Result<()> {
    //     let maybe_instruction = self.instructions.iter().nth(drone.program_counter as usize);
    //     if maybe_instruction.is_none() {
    //         return Err(Error::Error);
    //     }

    //     match maybe_instruction.unwrap() {
    //         ChangeDirection(x, y) => drone.direction = (*x, *y),
    //         Jump(idx) => drone.program_counter = *idx,
    //         Throttle(is_up) => drone.throttle = *is_up,
    //     }
    //     Ok(())
    // }

    // pub fn default() -> Self {
    //     Self {
    //         instructions: vec![
    //             Throttle(true),
    //             ChangeDirection(rand::random_range(-1f32..1f32), rand::random_range(-1f32..1f32)),
    //             Jump(2),
    //         ],
    //     }
    // }
}
