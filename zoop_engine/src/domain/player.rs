use bevy::prelude::*;

#[derive(Clone, Debug, Default, Component, Reflect)]
pub struct Player {
    pub handle: usize,
}