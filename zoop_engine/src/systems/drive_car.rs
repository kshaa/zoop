use crate::domain::car_body::CarMeta;

use crate::domain::game_config::GameConfig;
use crate::domain::ggrs_config::GGRSConfig;
use crate::domain::player::Player;
use crate::domain::tire::{TireMeta, TirePhysics};
use crate::logic::math::*;
use crate::logic::movement::*;

use crate::domain::controls::Controls;
use bevy::prelude::*;
use bevy_ggrs::*;
#[cfg(feature = "debug_lines")]
use bevy_prototype_debug_lines::*;
use bevy_rapier2d::prelude::*;
use bevy_sprite3d::Sprite3dParams;
use ggrs::InputStatus;
use crate::domain::spritesheets::SpriteSheets;
use crate::domain::trace::Trace;

pub fn store_car_positions(
    config: Res<GameConfig>,
    mut source_car_query: Query<(&mut CarMeta, &Transform, &Player), Without<TireMeta>>,
) {
    let mut car_query = source_car_query.iter_mut().collect::<Vec<_>>();
    car_query.sort_by_key(|(_, _, player)| player.handle);

    for (mut car_meta, car_transform, _) in car_query {
        car_meta.position_older = car_meta.position_old;
        car_meta.position_old = car_transform.translation;
        let velocity = car_meta.position_old.distance(car_meta.position_older);
        let delta = velocity - car_meta.velocity_smooth;
        let coefficient = config.camera_velocity_coefficient;
        let smoothed = (car_meta.velocity_smooth + delta * coefficient).abs();
        car_meta.velocity_smooth = smoothed;
    }
}

pub fn draw_drift_marks(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut source_tire_query: Query<(&mut TireMeta, &TirePhysics, &Transform, &Player), Without<CarMeta>>,
    spritesheets: Res<SpriteSheets>,
    mut sprite_params: Sprite3dParams,
) {
    let mut tire_query = source_tire_query.iter_mut().collect::<Vec<_>>();
    tire_query.sort_by_key(|(meta, _, _, player)| {
        (
            player.handle,
            if meta.is_front { 1 } else { 0 },
            if meta.is_right { 1 } else { 0 },
        )
    });

    for (mut tire_meta, tire_physics, tire_transform, _) in tire_query {
        tire_meta.position_older = tire_meta.position_old;
        let position = tire_transform.translation;
        tire_meta.position_old = position;
        let delta = tire_meta.last_trace.distance(position.clone());
        if delta > config.meters2pix(1.0) && tire_physics.drift_leftover != 0.0 {
            tire_meta.last_trace = position;
            commands.spawn(Trace::build(
                &spritesheets,
                &mut sprite_params,
                config.pixels_per_meter,
                tire_transform.clone()
            ));
        }
    }
}

pub fn drive_car(
    config: Res<GameConfig>,
    inputs: Option<Res<PlayerInputs<GGRSConfig>>>,
    fallback_inputs: Res<ButtonInput<KeyCode>>,
    mut source_car_query: Query<(&Transform, &Player), Without<TireMeta>>,
    mut source_tire_query: Query<
        (
            &mut TirePhysics,
            &TireMeta,
            &mut Transform,
            &Velocity,
            &mut ExternalForce,
            &mut ExternalImpulse,
            &Player,
        ),
        Without<CarMeta>,
    >,
) {
    // Sort queries for more determinism
    let mut car_query = source_car_query.iter_mut().collect::<Vec<_>>();
    car_query.sort_by_key(|(_, player)| player.handle);

    let mut tire_query = source_tire_query.iter_mut().collect::<Vec<_>>();
    tire_query.sort_by_key(|(_, meta, _, _, _, _, player)| {
        (
            player.handle,
            if meta.is_front { 1 } else { 0 },
            if meta.is_right { 1 } else { 0 },
        )
    });

    for (
        mut tire_physics,
        tire_meta,
        mut transform,
        velocity,
        mut forcable,
        mut impulsable,
        tire_player,
    ) in tire_query
    {
        let (game_input, _input_status) = if config.network.is_some() {
            inputs.as_ref().unwrap()[tire_player.handle]
        } else {
            (
                Controls::for_nth_player(&fallback_inputs, tire_player.handle),
                InputStatus::Confirmed,
            )
        };

        let controls = game_input;
        let original_tire_angle = tire_physics.angle;

        let (car_transform, _) = car_query
            .iter()
            .find(|(_, car_player)| car_player.handle == tire_player.handle)
            .unwrap();
        let (_, _, car_rotation) = car_transform.rotation.to_euler(EulerRot::XYZ);
        let tire_rotation = car_rotation + original_tire_angle;
        let tire_direction = Vec2::from_angle(tire_rotation + deg2rad(90.0)).normalize_or_zero();
        let direction_velocity = velocity.linvel.dot(tire_direction);

        // Apply drift leftover
        let is_drifting = velocity.linvel.length() > config.drift_velocity;
        if is_drifting {
            tire_physics.drift_leftover = 1.0;
        } else if tire_physics.drift_leftover != 0.0 {
            tire_physics.drift_leftover =
                (tire_physics.drift_leftover - config.drift_loss_per_tick).max(0.0);
        }

        // Apply controls to tire angle
        tire_physics.angle = tire_physics.angle
            + tire_angle_change(
                tire_meta,
                tire_physics.as_ref(),
                &controls,
                config.tire_min_angle,
                config.tire_max_angle,
                config.tire_rotation_per_tick,
                config.tire_rotation_velocity_reduction_coefficient,
                config.tire_rotation_step_reduction_coefficient,
                direction_velocity,
            );

        // Apply tire angle to tire rotation transform
        transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, tire_rotation);

        // Apply tire acceleration
        forcable.force = tire_direction
            * tire_acceleration(
                &tire_meta,
                &controls,
                config.pixels_per_meter,
                direction_velocity,
                config.tire_acceleration_force,
                config.tire_reversing_force,
                config.tire_breaking_force,
                config.parking_force,
            );

        // Apply friction
        let friction_impulse = tire_friction_impulse(
            is_drifting,
            tire_physics.drift_leftover,
            config.tire_friction_force,
            &tire_direction,
            &velocity.linvel,
        );
        impulsable.impulse += friction_impulse;
    }
}
