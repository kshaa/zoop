use crate::domain::car_body::CarMeta;
use crate::domain::frames::{log_confirmed_frame, log_start_frame, update_current_session_frame, update_rollback_status, CurrentSessionFrame, RollbackStatus};

use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_ggrs::*;
#[cfg(feature = "world_debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(feature = "debug_lines")]
use bevy_prototype_debug_lines::*;
use bevy_rapier2d::prelude::*;
use bevy_sprite3d::{Sprite3dPlugin};
use ggrs::InputStatus;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};

use crate::domain::colors::*;
use crate::domain::controls::*;
use crate::domain::game_config::GameConfig;
use crate::domain::game_readiness::GameReadiness;
use crate::domain::game_set::*;
use crate::domain::ggrs_config::GGRSConfig;
use crate::domain::player::Player;
use crate::domain::spawn::*;
use crate::domain::spritesheets::SpriteSheets;
use crate::domain::tire::TireMeta;
use crate::systems::build_network::*;
use crate::systems::drive_car::*;
use crate::systems::manage_scene::*;
use crate::systems::rollback_rapier_context::*;
use crate::systems::save_rapier_context::*;

pub fn build_game(game: &mut App, config: GameConfig) {
    // Log panics in browser console
    #[cfg(target_arch = "wasm32")]
    #[cfg(feature = "console_errors")]
    {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());
    }

    info!("Starting game with config {:?}", config);

    // Pre-spawn entities which will be re-used as game entities
    // for some reason Rapier requires these to be deterministic
    let _ = game
        .world
        .spawn_batch((0..101).map(DeterministicSpawnBundle::new))
        .collect::<Vec<Entity>>();

    // Generic game resources
    game.insert_resource(config.clone())
        .insert_resource(ClearColor(ZOOP_DARK_GRAY));

    // Default Bevy plugins
    game.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: config.canvas_selector.clone(),
            ..default()
        }),
        ..default()
    }));

    // For following camera
    game.add_plugins(LookTransformPlugin);
    game.add_systems(Update, move_camera_system);

    // Physics plugin
    game.insert_resource(config.rapier_config());
    game.add_plugins(
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(config.pixels_per_meter)
            .with_default_system_setup(false),
    );

    // Debug line renderer
    #[cfg(feature = "debug_lines")]
    game.add_plugins(DebugLinesPlugin::default());

    // Debug physics renderer
    #[cfg(feature = "rapier_debug_physics")]
    game.add_plugins(RapierDebugRenderPlugin::default());

    // Debug world inspector
    #[cfg(feature = "world_debug")]
    game.add_plugins(WorldInspectorPlugin::new());

    // Add 3d sprites
    game.add_plugins(Sprite3dPlugin);

    // Game readiness
    game.insert_resource(CurrentSessionFrame::default());
    game.insert_resource(RollbackStatus::default());
    game.init_state::<GameReadiness>();

    // physics toggling
    game.insert_resource(EnablePhysicsAfter::with_default_offset(
        0,
        config.fps as i32,
        config.load_seconds as i32,
    ));
    game.insert_resource(PhysicsEnabled::default());

    // Reset rapier
    game.add_systems(Startup, reset_rapier);

    // Init game state
    let state = init_scene(&config);
    game.insert_resource(state);
    game.add_systems(Startup, init_materials);
    game.add_systems(Update, await_assets.run_if(in_state(GameReadiness::LoadingAssets)));
    game.add_systems(Update, setup_scene.run_if(in_state(GameReadiness::LoadingScene)));

    // Define loading logic
    game.insert_resource(SpriteSheets::default());
    game.add_systems(Startup,
        |asset_server: Res<AssetServer>, mut spritesheets: ResMut<SpriteSheets>| {
            let car: Handle<Image> = asset_server.load("car.png");
            let tire: Handle<Image> = asset_server.load("tire.png");
            let trace: Handle<Image> = asset_server.load("trace.png");
            let building: Handle<Gltf> = asset_server.load("building.glb");
            spritesheets.car = car.clone();
            spritesheets.tire = tire.clone();
            spritesheets.trace = trace.clone();
            spritesheets.building = building.clone();
        },
    );

    // Define game logic schedule
    let game_schedule_label = GgrsSchedule;

    // Configure networking
    info!("Building game loop");
    if config.network.is_some() {
        // Init network and configure schedule
        build_network(game, &config);
    } else {
        // Manually attach game logic schedule
        let mut schedule = Schedule::default();
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Error,
            ..default()
        });
        game.add_schedule(schedule);
        // Add fixed schedule runner
        game.add_systems(FixedUpdate, (manual_frame_advance,));
        game.insert_resource(Time::<Fixed>::from_seconds(1.0 / (config.fps as f64)));
    }

    // Construct game logic schedule
    let game_schedule = game.get_schedule_mut(game_schedule_label).unwrap();
    game_schedule
        .configure_sets(
            (
                Rollback,
                Game,
                PhysicsSet::SyncBackend,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback,
                SaveAndChecksum,
            ).chain()
        )
        .add_systems(
            (
                log_start_frame,
                update_current_session_frame,
                log_confirmed_frame,
                update_rollback_status,
                toggle_physics.run_if(in_state(GameReadiness::Ready)),
                rollback_rapier_context,
                // Make sure to flush everything before we apply our game logic.
                apply_deferred,
            )
                .chain()
                .in_set(Rollback),
        );

    if config.network.is_some() {
        game_schedule.add_systems(
            (
                // destroy_scene,
                // setup_scene,
                store_car_positions.before(drive_car),
                draw_drift_marks.before(drive_car).after(store_car_positions),
                drive_car,
                // The `frame_validator` relies on the execution of `apply_inputs` and must come after.
                // It could happen anywhere else, I just stuck it here to be clear.
                // If this is causing your game to quit, you have a bug!
                // Make sure to flush everything before Rapier syncs
                apply_deferred,
            )
                .chain()
                .in_set(Game),
        );
    } else {
        game_schedule.add_systems(
            (
                store_car_positions.before(drive_car),
                draw_drift_marks.before(drive_car).after(store_car_positions),
                drive_car
            )
                .chain()
                .in_set(Game),
        );
    }

    game_schedule
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                .in_set(PhysicsSet::SyncBackend),
        )
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                .in_set(PhysicsSet::StepSimulation),
        )
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                .in_set(PhysicsSet::Writeback),
        )
        .add_systems(
            (
                save_rapier_context, // This must execute after writeback to store the RapierContext
                apply_deferred, // Flushing again
            )
                .chain()
                .in_set(SaveAndChecksum),
        );

    // Scene setup
    game.add_systems(Startup, setup_graphics);
}

fn setup_graphics(config: Res<GameConfig>, mut commands: Commands) {
    let eye = Vec3 {
        x: 0.0,
        y: 0.0,
        z: config.default_camera_height,
    };
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    commands
        .spawn(LookTransformBundle {
            transform: LookTransform::new(eye, target, up),
            smoother: Smoother::new(0.8), // Value between 0.0 and 1.0, higher is smoother.
        })
        .insert(Camera3dBundle {
            transform: Transform::from_translation(eye).looking_at(target, up),
            ..default()
        });
}
use crate::domain::controls::Controls;

fn move_camera_system(
    config: Res<GameConfig>,
    mut cameras: Query<&mut LookTransform>,
    source_car_query: Query<(&CarMeta, &Transform, &Player), Without<TireMeta>>,
    inputs: Option<Res<PlayerInputs<GGRSConfig>>>,
    fallback_inputs: Res<ButtonInput<KeyCode>>,
) {
    let following_car_index = 0;

    let (game_input, _): (Controls, InputStatus) = if config.network.is_some() {  
        let network_inputs = inputs.as_ref();
        match network_inputs {
            Some(inputs) => inputs[following_car_index],
            None => (Controls::empty(), InputStatus::Predicted)
        }
    } else {
        (Controls::for_nth_player(&fallback_inputs, following_car_index), InputStatus::Confirmed)
    };

    let camera_height =
        if (game_input.parking()) { config.parking_camera_height }
        else { config.default_camera_height };

    let (position, velocity) = source_car_query
        .into_iter()
        .find(|(_, _, p)| p.handle == following_car_index)
        .map(|(m, t, _)| (t.translation, m.velocity_smooth))
        .unwrap_or((Vec3::ZERO, 0.0));

    // Later, another system will update the `Transform` and apply smoothing automatically.
    for mut c in cameras.iter_mut() {
        c.eye.x = position.x;
        c.eye.y = position.y;
        c.eye.z = camera_height + velocity * 70.0;
        c.target = position;
    }
}

fn rapier_stub() {}

fn rapier_stub2() {}

fn manual_frame_advance(world: &mut World) {
    world.run_schedule(GgrsSchedule);
}
