use crate::domain::checksum::fletcher16;
use crate::domain::frames::*;
use crate::domain::game_config::GameConfig;
use crate::domain::rapier_rollback_state::RapierRollbackState;
use bevy::prelude::*;
use bevy_rapier2d::plugin::RapierContext;

pub fn save_rapier_context(
    config: Res<GameConfig>,
    mut game_state: ResMut<RapierRollbackState>,
    rapier: Res<RapierContext>,
) {
    // This serializes our context every frame.  It's not great, but works to
    // integrate the two plugins.  To do less of it, we would need to change
    // bevy_ggrs to serialize arbitrary structs like this one in addition to
    // component tracking.  If you need this to happen less, I'd recommend not
    // using the plugin and implementing GGRS yourself.
    if let Ok(context_bytes) = bincode::serialize(rapier.as_ref()) {
        debug!("Context hash before save: {}", game_state.rapier_checksum);
        game_state.rapier_checksum = fletcher16(&context_bytes);
        game_state.rapier_state = Some(context_bytes);
        debug!("Context hash after save: {}", game_state.rapier_checksum);
    }
}
