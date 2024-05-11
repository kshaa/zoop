use crate::domain::controls::Controls;
use crate::domain::ggrs_config::GGRSConfig;
use crate::systems::rollback_rapier_context::PhysicsEnabled;
use bevy::utils::HashMap;
use bevy::prelude::*;
use bevy_ggrs::LocalInputs;
use bevy_ggrs::LocalPlayers;

pub fn read_network_controls(
    mut commands: Commands,
    local_players: Res<LocalPlayers>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    physics_enabled: Res<PhysicsEnabled>
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let controls = if !physics_enabled.0 {
            Controls::empty()
        } else {
            // TODO: This should support more control configurations for local-only play
            Controls::from_wasd(keyboard_input.as_ref())
        };
        
        local_inputs.insert(*handle, controls);
    }

    commands.insert_resource(LocalInputs::<GGRSConfig>(local_inputs));
}
