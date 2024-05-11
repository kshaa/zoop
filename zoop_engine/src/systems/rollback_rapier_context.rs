use crate::domain::checksum::*;
use crate::domain::frames::RollbackStatus;
use crate::domain::rapier_rollback_state::RapierRollbackState;
use bevy::prelude::*;
use bevy_ggrs::ggrs::Frame;
use bevy_ggrs::RollbackFrameCount;
use bevy_rapier2d::plugin::RapierContext;
use bevy_rapier2d::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Resource, Hash, Reflect)]
#[reflect(Hash)]
pub struct PhysicsEnabled(pub bool);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Resource, Hash, Reflect)]
#[reflect(Hash, Resource, PartialEq)]
pub struct EnablePhysicsAfter {
    pub start: Frame,
    pub end: Frame,
}

impl Default for EnablePhysicsAfter {
    fn default() -> Self {
        Self::with_default_offset(0, 0, 0)
    }
}

impl EnablePhysicsAfter {
    pub fn new(start: Frame, end: Frame) -> Self {
        info!("Enabling after {:?},{:?}", start, end);
        Self { start, end }
    }

    pub fn with_default_offset(offset: Frame, fps: i32, load_seconds: i32) -> Self {
        Self::new(offset, offset + (fps * load_seconds) as i32)
    }

    pub fn is_enabled(&self, frame: Frame) -> bool {
        // Since the starting frame is calculated at the end,
        // when we rollback to the start frame we will have the enable after
        // resource of that frame it was created as a result of, which is wrong.
        // assume that 1 frame is actually good and should not be ignored
        !(self.start < frame && frame < self.end)
    }
}

pub fn toggle_physics(
    enable_physics_after: Res<EnablePhysicsAfter>,
    current_frame: Res<RollbackFrameCount>,
    mut physics_enabled: ResMut<PhysicsEnabled>,
    mut config: ResMut<RapierConfiguration>,
) {
    debug!(
        "Physics on frame {:?} {:?} {:?}",
        current_frame.0, physics_enabled.0, enable_physics_after
    );
    let should_activate = enable_physics_after.is_enabled(current_frame.0);
    if physics_enabled.0 != should_activate {
        info!(
            "Toggling physics on frame {:?}: {:?} -> {:?}",
            current_frame.0, physics_enabled.0, should_activate
        );
        physics_enabled.0 = should_activate;
    }

    config.physics_pipeline_active = physics_enabled.0;
}

pub fn reset_rapier(
    mut commands: Commands,
    mut rapier: ResMut<RapierContext>,
    collider_handles: Query<Entity, With<RapierColliderHandle>>,
    rb_handles: Query<Entity, With<RapierRigidBodyHandle>>,
) {
    // You might be wondering:  why is this here?  What purpose does it serve?
    // In just resets everything on startup!
    // Yes.  But this bad boy right here is a good system you can use to reset
    // Rapier whenever you please in your game (e.g., after a game ends or
    // between rounds).  It isn't quite a nuclear option, but a rollbackable one!

    // Force rapier to reload everything
    for e in collider_handles.iter() {
        commands.entity(e).remove::<RapierColliderHandle>();
    }
    for e in rb_handles.iter() {
        commands.entity(e).remove::<RapierRigidBodyHandle>();
    }

    // Re-initialize everything we overwrite with default values
    let context = RapierContext::default();
    rapier.bodies = context.bodies;
    rapier.colliders = context.colliders;
    rapier.broad_phase = context.broad_phase;
    rapier.narrow_phase = context.narrow_phase;
    rapier.ccd_solver = context.ccd_solver;
    rapier.impulse_joints = context.impulse_joints;
    rapier.integration_parameters = context.integration_parameters;
    rapier.islands = context.islands;
    rapier.multibody_joints = context.multibody_joints;
    rapier.pipeline = context.pipeline;
    rapier.query_pipeline = context.query_pipeline;

    // Add a bit more CCD
    // This is objectively just something that could be setup once, but we did
    // just wholesale overwrite this anyway.  I think you can just not override
    // integration_parameters above, but where's the fun in that?
    rapier.integration_parameters.max_ccd_substeps = 5;

    // Serialize our "blank" slate for frame 0.
    // This is actually important because it is possible to rollback to this!
    if let Ok(context_bytes) = bincode::serialize(rapier.as_ref()) {
        let rapier_checksum = fletcher16(&context_bytes);
        info!("Context hash at init: {}", rapier_checksum);

        commands.insert_resource(RapierRollbackState {
            rapier_state: Some(context_bytes),
            rapier_checksum,
        })
    } else {
        commands.insert_resource(RapierRollbackState::default());
    }
}

pub fn rollback_rapier_context(
    rollback_status: Res<RollbackStatus>,
    game_state: Res<RapierRollbackState>,
    mut rapier: ResMut<RapierContext>,
) {
    let mut checksum = game_state.rapier_checksum;
    debug!("Context pre-hash at start: {:?}", checksum);

    // Serialize our physics state for hashing, to display the state in-flight.
    // This should not be necessary for this demo to work, as we will do the
    // real checksum during `save_game_state` at the end of the pipeline.
    if let Ok(context_bytes) = bincode::serialize(rapier.as_ref()) {
        checksum = fletcher16(&context_bytes);
        debug!("Context hash at start: {}", checksum);
    }

    // Only restore our state if we are in a rollback.  This step is *critical*.
    // Only doing this during rollbacks saves us a step every frame.  Here, we
    // also do not allow rollback to frame 0.  Physics state is already correct
    // in this case.  This prevents lagged clients from getting immediate desync
    // and is entirely a hack since we don't enable physics until later anyway.
    //
    // You can also test that desync detection is working by disabling:
    // if false {
    if rollback_status.is_rollback && rollback_status.rollback_frame > 1 {
        if let Some(state_context) = game_state.rapier_state.as_ref() {
            if let Ok(context) = bincode::deserialize::<RapierContext>(state_context) {
                // commands.insert_resource(context);
                // *rapier = context;

                // Inserting or replacing directly seems to screw up some of the
                // crate-only properties.  So, we'll copy over each public
                // property instead.
                rapier.bodies = context.bodies;
                rapier.broad_phase = context.broad_phase;
                rapier.ccd_solver = context.ccd_solver;
                rapier.colliders = context.colliders;
                rapier.impulse_joints = context.impulse_joints;
                rapier.integration_parameters = context.integration_parameters;
                rapier.islands = context.islands;
                rapier.multibody_joints = context.multibody_joints;
                rapier.narrow_phase = context.narrow_phase;
                rapier.query_pipeline = context.query_pipeline;

                // pipeline is not serialized
                // rapier.pipeline = context.pipeline;
            }
        }

        // Again, not necessary for the demo, just to show the rollback changes
        // as they occur.
        if let Ok(context_bytes) = bincode::serialize(rapier.as_ref()) {
            debug!(
                "Context hash after rollback: {}",
                fletcher16(&context_bytes)
            );
        }
    }
}
