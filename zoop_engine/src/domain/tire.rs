use crate::domain::game_state::GameTire;
use crate::domain::player::Player;
use crate::domain::spritesheets::SpriteSheets;
use bevy::core::Name;
use bevy::math::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dParams};

#[derive(Copy, Clone, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct TireMeta {
    pub is_front: bool,
    pub is_right: bool,
    pub position_old: Vec3,
    pub position_older: Vec3,
    pub last_trace: Vec3,
}

#[derive(Copy, Clone, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct TirePhysics {
    pub angle: f32,
    pub drift_leftover: f32,
}

#[derive(Bundle)]
pub struct Tire {
    meta: TireMeta,
    phyics: TirePhysics,
    name: Name,
    rigid_body: RigidBody,
    damping: Damping,
    velocity: Velocity,
    read_mass_properties: ReadMassProperties,
    // mass: AdditionalMassProperties,
    sleeping: Sleeping,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
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

#[allow(dead_code)]
pub struct TireEntities {
    pub top_right: Entity,
    pub top_left: Entity,
    pub bottom_right: Entity,
    pub bottom_left: Entity,
}

impl Tire {
    pub fn build(
        spritesheets: &SpriteSheets,
        sprite_params: &mut Sprite3dParams,
        pixels_per_meter: f32,
        player: Player,
        is_front: bool,
        is_right: bool,
        half_size: Vec2,
        car_title: String,
        _color: Color,
        damping: Damping,
        physics: GameTire,
    ) -> Tire {
        let front_title = if is_front { "F" } else { "B" };
        let side_title = if is_right { "R" } else { "L" };
        let position = physics.entity_physics.transform.translation;

        Tire {
            meta: TireMeta {
                is_front,
                is_right,
                position_old: position.clone(),
                position_older: position.clone(),
                last_trace: position.clone()
            },
            phyics: physics.tire_physics,
            name: Name::new(format!(
                "Tire {}{} for {}",
                front_title, side_title, car_title
            )),
            rigid_body: RigidBody::Dynamic,
            damping,
            velocity: physics.entity_physics.velocity,
            read_mass_properties: physics.entity_physics.mass,
            // mass: AdditionalMassProperties::Mass(0.001),
            sleeping: Sleeping::disabled(),
            external_force: physics.entity_physics.force,
            external_impulse: physics.entity_physics.impulse,
            collider: Collider::cuboid(half_size.x, half_size.y),
            collider_scale: ColliderScale::Absolute(Vec2::new(1., 1.)),
            locked_axes: LockedAxes::default(),
            restitution: Restitution::default(),
            friction: Friction::default(),
            active_events: ActiveEvents::empty(),
            ccd: Ccd::disabled(),
            collision_groups: CollisionGroups::new(Group::NONE, Group::NONE),
            sprite3d: Sprite3d {
                image: spritesheets.tire.clone(),
                pixels_per_metre: 500.0 / pixels_per_meter,
                // custom_size: Some(half_size * 2.0),
                // partial_alpha: true,
                unlit: true,
                transform: physics.entity_physics.transform,
                ..default()
            }
            .bundle(sprite_params),
            player,
        }
    }
}
