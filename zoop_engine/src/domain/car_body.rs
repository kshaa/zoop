use crate::domain::game_state::*;
use crate::domain::player::Player;
use crate::domain::spritesheets::SpriteSheets;

use bevy::core::Name;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_sprite3d::*;

#[derive(Component)]
pub struct CarMeta {
    pub half_size: Vec2,
    pub position_old: Vec3,
    pub position_older: Vec3,
    pub velocity_smooth: f32,
}

#[derive(Bundle)]
pub struct CarBody {
    meta: CarMeta,
    name: Name,
    rigid_body: RigidBody,
    rigid_mass: ReadMassProperties,
    // mass: AdditionalMassProperties,
    sleep: Sleeping,
    force: ExternalForce,
    impulse: ExternalImpulse,
    velocity: Velocity,
    collider: Collider,
    collider_scale: ColliderScale,
    locked_axes: LockedAxes,
    restitution: Restitution,
    friction: Friction,
    active_events: ActiveEvents,
    ccd: Ccd,
    collision_groups: CollisionGroups,
    sprite3d: Sprite3dBundle,
    player: Player,
}

impl CarBody {
    pub fn build(
        spritesheets: &SpriteSheets,
        sprite_params: &mut Sprite3dParams,
        pixels_per_meter: f32,
        car_title: String,
        half_size: Vec2,
        radius: f32,
        player: Player,
        _color: Color,
        physics: EntityPhysics,
    ) -> CarBody {
        let position = physics.transform.translation;
        CarBody {
            meta: CarMeta {
                half_size,
                position_old: position,
                position_older: position,
                velocity_smooth: 0.0,
            },
            rigid_body: RigidBody::Dynamic,
            name: Name::new(car_title),
            rigid_mass: physics.mass,
            // mass: AdditionalMassProperties::Mass(0.001),
            sleep: Sleeping::disabled(),
            force: physics.force,
            impulse: physics.impulse,
            collider: Collider::round_cuboid(half_size.x, half_size.y, radius),
            collider_scale: ColliderScale::Absolute(Vec2::new(1., 1.)),
            locked_axes: LockedAxes::default(),
            restitution: Restitution::default(),
            friction: Friction::default(),
            active_events: ActiveEvents::empty(),
            ccd: Ccd::disabled(),
            collision_groups: CollisionGroups::new(Group::GROUP_1, Group::GROUP_1),
            velocity: physics.velocity,
            sprite3d: Sprite3d {
                image: spritesheets.car.clone(),
                pixels_per_metre: 250.0 / pixels_per_meter,
                // partial_alpha: true,
                unlit: true,
                transform: physics
                    .transform
                    .with_translation(position + Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            }
            .bundle(sprite_params),
            player,
        }
    }
}
