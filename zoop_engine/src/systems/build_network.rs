use crate::domain::game_config::GameConfig;
use crate::domain::game_state::GameState;
use crate::domain::ggrs_config::GGRSConfig;
use crate::domain::rapier_rollback_state::RapierRollbackState;
use crate::services::websocket::*;
use crate::systems::read_network_controls::read_network_controls;
use crate::systems::rollback_rapier_context::EnablePhysicsAfter;
use bevy::prelude::*;
use bevy_ggrs::*;
use bevy_rapier2d::dynamics::{Sleeping, Velocity};
use ggrs::*;

pub fn build_network(game: &mut App, config: &GameConfig) {
    let session = start_network_session(&config);
    build_ggrs(game, config);
    game.insert_resource(Session::P2P(session));
}

pub fn build_ggrs(game: &mut App, config: &GameConfig) {
    game.add_plugins(GgrsPlugin::<GGRSConfig>::default())
        // define frequency of rollback game logic update
        .set_rollback_schedule_fps(usize::from(config.fps))
        // define system that returns inputs given a player handle, so GGRS can send the inputs around
        .add_systems(ReadInputs, read_network_controls)
        // register types of components AND resources you want to be rolled back
        // TODO: Refactor to rollback_resource_with_clone and remove my own fake reflect & clone code
        .checksum_resource_with_hash::<RapierRollbackState>()
        .rollback_resource_with_reflect::<RapierRollbackState>()
        // Store everything that Rapier updates in its Writeback stage
        .rollback_component_with_reflect::<GlobalTransform>()
        .rollback_component_with_reflect::<Transform>()
        .rollback_component_with_reflect::<Velocity>()
        .rollback_component_with_reflect::<Sleeping>()
        // Game stuff
        .rollback_resource_with_reflect::<EnablePhysicsAfter>()
        // # physics
        // .register_rollback_component::<Velocity>()
        // .register_rollback_component::<AdditionalMassProperties>()
        // .register_rollback_component::<ReadMassProperties>()
        // // .register_rollback_component::<MassProperties>()
        // .register_rollback_component::<LockedAxes>()
        // .register_rollback_component::<ExternalForce>()
        // .register_rollback_component::<ExternalImpulse>()
        // .register_rollback_component::<Sleeping>()
        // .register_rollback_component::<Damping>()
        // .register_rollback_component::<Dominance>()
        // .register_rollback_component::<Ccd>()
        // .register_rollback_component::<GravityScale>()
        // .register_rollback_component::<CollidingEntities>()
        // .register_rollback_component::<Sensor>()
        // .register_rollback_component::<Friction>()
        // .register_rollback_component::<Restitution>()
        // .register_rollback_component::<CollisionGroups>()
        // .register_rollback_component::<SolverGroups>()
        // .register_rollback_component::<ContactForceEventThreshold>()
        // .register_rollback_component::<Group>()
        .rollback_resource_with_reflect::<GameState>();
        // # bevy
        // .register_rollback_component::<Transform>()
        // # game
        // .register_rollback_component::<TireMeta>()
        // these systems will be executed as part of the advance frame update
}

pub fn start_network_session(config: &GameConfig) -> P2PSession<GGRSConfig> {
    // Create a GGRS session
    let mut session_builder = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(config.players.len())
        .with_desync_detection_mode(ggrs::DesyncDetection::On { interval: 10 }) // (optional) set how often to exchange state checksums
        .with_max_prediction_window(12) // (optional) set max prediction window
        .expect("Invalid prediction window")
        .with_input_delay(3); // (optional) set input delay for the local player

    // Add players
    for (i, network_player) in config.players.iter().enumerate() {
        // local player
        if network_player.is_local {
            session_builder = session_builder.add_player(PlayerType::Local, i).unwrap();
        } else {
            // remote players
            session_builder = session_builder
                .add_player(PlayerType::Remote(network_player.id.clone()), i)
                .unwrap();
        }
    }

    // Start the GGRS session
    let room_address = config.game_room_address().unwrap().unwrap();
    let socket = NonBlockingWebSocket::connect(room_address.to_string()).unwrap();

    session_builder.start_p2p_session(socket).unwrap()
}
