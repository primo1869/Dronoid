use crate::{
    Result,
    entity::Instruction::{ChangeDirection, Jump, Throttle},
    error::Error,
};

pub(crate) enum Building {
    SpawnBeacon(SpawnBeacon),
    Factory(Factory),
}

pub(crate) trait Discover {
    fn radius(&self) -> f32;
}

pub(crate) trait Cycle {
    fn cycle(&mut self, delta: f32);
}

pub(crate) enum Instruction {
    Jump(u16),
    Throttle(bool),
    ChangeDirection(f32, f32),
}

pub(crate) struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    fn exec(&self, drone: &mut Drone) -> Result<()> {
        let maybe_instruction = self.instructions.iter().nth(drone.program_counter as usize);
        if maybe_instruction.is_none() {
            return Err(Error::Error);
        }

        match maybe_instruction.unwrap() {
            ChangeDirection(x, y) => drone.direction = (*x, *y),
            Jump(idx) => drone.program_counter = *idx,
            Throttle(is_up) => drone.throttle = *is_up,
        }
        Ok(())
    }

    fn default() -> Self {
        Self {
            instructions: vec![
                Throttle(true),
                ChangeDirection(
                    rand::random_range(-1f32..1f32),
                    rand::random_range(-1f32..1f32),
                ),
                Jump(2),
            ],
        }
    }
}

pub(crate) struct SpawnBeacon;

impl Discover for SpawnBeacon {
    fn radius(&self) -> f32 {
        25.
    }
}

pub(crate) struct Factory {
    auto_spawn: bool,
    program: Program,
    drones: Vec<Drone>,
}

impl Factory {
    fn new() -> Self {
        Self {
            auto_spawn: false,
            drones: Vec::new(),
            program: Program::default(),
        }
    }
}

impl Cycle for Factory {
    fn cycle(&mut self, delta: f32) {
        for drone in &self.drones {}
    }
}

impl Discover for Factory {
    fn radius(&self) -> f32 {
        50.
    }
}

pub(crate) struct Drone {
    direction: (f32, f32),
    throttle: bool,
    program_counter: u16,
}

impl Drone {
    fn new() -> Self {
        Self {
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
