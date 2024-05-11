use bevy::ecs::schedule::{ScheduleLabel, SystemSet};
use std::{fmt::Debug, hash::Hash};

#[derive(ScheduleLabel, SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rollback;

#[derive(ScheduleLabel, SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Game;

#[derive(ScheduleLabel, SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SaveAndChecksum;
