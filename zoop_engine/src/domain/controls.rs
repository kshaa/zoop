use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

const INPUT_ACCELERATE: u16 = 1 << 0;
const INPUT_REVERSE: u16 = 1 << 1;
const INPUT_BREAK: u16 = 1 << 2;
const INPUT_STEER_RIGHT: u16 = 1 << 3;
const INPUT_STEER_LEFT: u16 = 1 << 4;
const INPUT_PARK: u16 = 1 << 5;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct Controls {
    pub input: u16,
}
impl Controls {
    pub fn accelerating(&self) -> bool {
        (self.input & INPUT_ACCELERATE) != 0
    }
    pub fn reversing(&self) -> bool {
        (self.input & INPUT_REVERSE) != 0
    }
    pub fn breaking(&self) -> bool {
        (self.input & INPUT_BREAK) != 0
    }
    pub fn steering_right(&self) -> bool {
        (self.input & INPUT_STEER_RIGHT) != 0
    }
    pub fn steering_left(&self) -> bool {
        (self.input & INPUT_STEER_LEFT) != 0
    }
    pub fn parking(&self) -> bool {
        (self.input & INPUT_PARK) != 0
    }

    pub fn steering_any(&self) -> bool {
        self.steering_right() || self.steering_left()
    }

    pub fn from_keys(
        input: &ButtonInput<KeyCode>,
        accelerator: KeyCode,
        reverser: KeyCode,
        breaker: KeyCode,
        steer_right: KeyCode,
        steer_left: KeyCode,
        park: KeyCode
    ) -> Controls {
        let mut serialized: u16 = 0;

        if input.pressed(accelerator) {
            serialized |= INPUT_ACCELERATE
        }
        if input.pressed(reverser) {
            serialized |= INPUT_REVERSE
        }
        if input.pressed(breaker) {
            serialized |= INPUT_BREAK
        }
        if input.pressed(steer_right) {
            serialized |= INPUT_STEER_RIGHT
        }
        if input.pressed(steer_left) {
            serialized |= INPUT_STEER_LEFT
        }
        if input.pressed(park) {
            serialized |= INPUT_PARK
        }

        Controls {
            input: serialized,
        }
    }

    pub fn empty() -> Controls {
        Controls {
            input: 0,
        }
    }

    pub fn from_wasd(
        input: &ButtonInput<KeyCode>,
    ) -> Controls {
        Controls::from_keys(
            input,
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyC,
            KeyCode::KeyD,
            KeyCode::KeyA,
            KeyCode::KeyV,
        )
    }

    pub fn from_ijkl(
        input: &ButtonInput<KeyCode>,
    ) -> Controls {
        Controls::from_keys(
            input,
            KeyCode::KeyI,
            KeyCode::KeyK,
            KeyCode::KeyN,
            KeyCode::KeyL,
            KeyCode::KeyJ,
            KeyCode::KeyB
        )
    }

    pub fn for_nth_player(input: &ButtonInput<KeyCode>, n: usize) -> Controls {
        if n == 0 {
            Controls::from_wasd(input)
        } else {
            Controls::from_ijkl(input)
        }
    }
}
