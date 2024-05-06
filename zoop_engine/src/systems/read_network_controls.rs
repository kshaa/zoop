use crate::domain::controls::Controls;
use crate::domain::desync::*;
use crate::domain::frames::*;
use crate::systems::rollback_rapier_context::PhysicsEnabled;
use bevy::prelude::*;
use ggrs::PlayerHandle;

pub fn read_network_controls(
    _handle: In<PlayerHandle>,
    keyboard_input: Res<Input<KeyCode>>,
    physics_enabled: Res<PhysicsEnabled>,
    mut hashes: ResMut<FrameHashes>,
    validatable_frame: Res<ValidatableFrame>,
) -> Controls {
    let mut last_confirmed_frame = ggrs::NULL_FRAME;
    let mut last_confirmed_hash = 0;

    // Find a hash that we haven't sent yet.
    // This probably seems like overkill but we have to track a bunch anyway, we
    // might as well do our due diligence and inform our opponent of every hash
    // we have This may mean we ship them out of order.  The important thing is
    // we determine the desync *eventually* because that match is pretty much
    // invalidated without a state synchronization mechanism (which GGRS/GGPO
    // does not have out of the box.)
    for frame_hash in hashes.0.iter_mut() {
        // only send confirmed frames that have not yet been sent that are well past our max prediction window
        if frame_hash.confirmed
            && !frame_hash.sent
            && validatable_frame.is_validatable(frame_hash.frame)
        {
            trace!("Sending data {:?}", frame_hash);
            last_confirmed_frame = frame_hash.frame;
            last_confirmed_hash = frame_hash.rapier_checksum;
            frame_hash.sent = true;
        }
    }

    let controls = if !physics_enabled.0 {
        Controls::empty(last_confirmed_hash, last_confirmed_frame)
    } else {
        Controls::from_wasd(
            keyboard_input.as_ref(),
            last_confirmed_hash,
            last_confirmed_frame,
        )
    };

    trace!("Configured controls: {:?}", &controls);
    controls
}
