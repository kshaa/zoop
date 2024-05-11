use crate::domain::ggrs_config::GGRSConfig;
use bevy::prelude::*;
use bevy_ggrs::Session;
use bevy_ggrs::{ConfirmedFrameCount, RollbackFrameCount};
use ggrs::*;

/// Left outside of the rollback system to detect rollbacks
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct LastFrame(pub Frame);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct ConfirmedFrame(pub Frame);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct CurrentSessionFrame(pub Frame);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct CurrentFrame(pub Frame);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct ValidatableFrame(pub Frame);

impl Default for ValidatableFrame {
    fn default() -> Self {
        Self(std::i32::MIN)
    }
}

impl ValidatableFrame {
    pub fn is_validatable(&self, frame: Frame) -> bool {
        frame < self.0
    }
}

/// Should not be rolled back... obviously?
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct RollbackStatus {
    pub is_rollback: bool,
    pub is_replay: bool,
    pub rollback_frame: Frame,
    pub last_frame: Frame,
}

pub fn log_confirmed_frame(
    mut confirmed_frame: Res<ConfirmedFrameCount>
) {
    let confirmed_frame: i32 = (*confirmed_frame).into();
    debug!("confirmed frame: {}", confirmed_frame);
}

pub fn log_start_frame(mut current_frame: Res<ConfirmedFrameCount>) {
    let current_frame: i32 = (*current_frame).into();
    debug!("---- start frame {} ----", current_frame);
}

pub fn update_current_session_frame(
    mut current_session_frame: ResMut<CurrentSessionFrame>,
    current_frame: Res<RollbackFrameCount>,
    session: Option<Res<Session<GGRSConfig>>>,
) {
    let current_frame: i32 = (*current_frame).into();

    if let Some(session) = session {
        match &*session {
            Session::SyncTest(_) => current_session_frame.0 = current_frame,
            Session::P2P(s) => current_session_frame.0 = s.current_frame(),
            Session::Spectator(_) => current_session_frame.0 = current_frame,
        }
    }

    debug!("current session frame: {}", current_session_frame.0);
}

pub fn update_rollback_status(
    current_frame: Res<RollbackFrameCount>,
    current_session_frame: Res<CurrentSessionFrame>,
    mut rollback_status: ResMut<RollbackStatus>,
) {
    let current_frame: i32 = (*current_frame).into();

    // If the last frame is greater than the current frame, we have rolled back.
    // Same for equals, because it means our frame did not update!
    rollback_status.is_rollback = rollback_status.last_frame >= current_frame;
    rollback_status.is_replay =
        rollback_status.is_rollback || current_session_frame.0 > current_frame;

    if rollback_status.is_rollback {
        rollback_status.rollback_frame = current_frame;
        info!(
            "rollback on {} to {}",
            rollback_status.last_frame, rollback_status.rollback_frame,
        );
    }

    if rollback_status.is_replay {
        info!(
            "replay on {} of {}",
            current_session_frame.0, current_frame
        );
    }

    // I know this seems silly at first glance, but after we know we've entered
    // a rollback once, we have to resimulate all frames back to where we left
    // off... and there may be additional rollbacks that happen during that!
    rollback_status.last_frame = current_frame;
}
