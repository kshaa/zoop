use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use ggrs::*;

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

    pub last_confirmed_hash: u16,
    pub last_confirmed_frame: Frame,
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
        input: &Input<KeyCode>,
        accelerator: KeyCode,
        reverser: KeyCode,
        breaker: KeyCode,
        steer_right: KeyCode,
        steer_left: KeyCode,
        park: KeyCode,
        last_confirmed_hash: u16,
        last_confirmed_frame: Frame,
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
            last_confirmed_hash,
            last_confirmed_frame,
        }
    }

    pub fn empty(last_confirmed_hash: u16, last_confirmed_frame: Frame) -> Controls {
        Controls {
            input: 0,
            last_confirmed_hash,
            last_confirmed_frame,
        }
    }

    pub fn unknown() -> Controls {
        Controls::empty(0, 0)
    }

    pub fn from_wasd(
        input: &Input<KeyCode>,
        last_confirmed_hash: u16,
        last_confirmed_frame: Frame,
    ) -> Controls {
        Controls::from_keys(
            input,
            KeyCode::W,
            KeyCode::S,
            KeyCode::C,
            KeyCode::D,
            KeyCode::A,
            KeyCode::V,
            last_confirmed_hash,
            last_confirmed_frame,
        )
    }

    pub fn from_ijkl(
        input: &Input<KeyCode>,
        last_confirmed_hash: u16,
        last_confirmed_frame: Frame,
    ) -> Controls {
        Controls::from_keys(
            input,
            KeyCode::I,
            KeyCode::K,
            KeyCode::N,
            KeyCode::L,
            KeyCode::J,
            KeyCode::B,
            last_confirmed_hash,
            last_confirmed_frame,
        )
    }

    pub fn for_nth_player(input: &Input<KeyCode>, n: usize) -> Controls {
        let hash: u16 = 0;
        let frame: Frame = 0;
        if n == 0 {
            Controls::from_wasd(input, hash, frame)
        } else {
            Controls::from_ijkl(input, hash, frame)
        }
    }
}
