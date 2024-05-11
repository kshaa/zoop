use crate::domain::car::tire_position;
use crate::domain::player::Player;
use crate::domain::tire::TirePhysics;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Clone, Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct EntityPhysics {
    pub transform: Transform,
    pub velocity: Velocity,
    pub force: ExternalForce,
    pub impulse: ExternalImpulse,
    pub mass: ReadMassProperties,
}
impl EntityPhysics {
    pub fn of(
        transform: Transform,
        velocity: Velocity,
        force: ExternalForce,
        impulse: ExternalImpulse,
        mass: ReadMassProperties,
    ) -> EntityPhysics {
        EntityPhysics {
            transform,
            velocity,
            force,
            impulse,
            mass,
        }
    }
    pub fn fixed(position: Vec3) -> EntityPhysics {
        EntityPhysics {
            transform: Transform::from_translation(position),
            velocity: Velocity::zero(),
            force: ExternalForce::default(),
            impulse: ExternalImpulse::default(),
            mass: ReadMassProperties::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameTire {
    pub tire_physics: TirePhysics,
    pub entity_physics: EntityPhysics,
}
impl GameTire {
    #[allow(dead_code)]
    pub fn of(
        transform: Transform,
        velocity: Velocity,
        force: ExternalForce,
        impulse: ExternalImpulse,
        mass: ReadMassProperties,
        angle: f32,
    ) -> GameTire {
        GameTire {
            tire_physics: TirePhysics {
                angle,
                drift_leftover: 0.0,
            },
            entity_physics: EntityPhysics::of(transform, velocity, force, impulse, mass),
        }
    }
    pub fn fixed(position: Vec3) -> GameTire {
        GameTire {
            tire_physics: TirePhysics::default(),
            entity_physics: EntityPhysics::fixed(position),
        }
    }
}

#[derive(Clone, Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameBuilding {
    pub position: Vec3,
    pub stories: u32,
    pub is_tunnel: bool,
    pub half_size: f32,
}
impl GameBuilding {
    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.position.clone())
            .with_scale(Vec3::ONE * self.half_size)
    }

    pub fn of(position: Vec3, stories: u32, is_tunnel: bool, half_size: f32) -> GameBuilding {
        GameBuilding {
            position,
            stories,
            is_tunnel,
            half_size,
        }
    }
}

#[derive(Clone, Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameCar {
    pub tire_top_left: GameTire,
    pub tire_top_right: GameTire,
    pub tire_bottom_left: GameTire,
    pub tire_bottom_right: GameTire,
    pub physics: EntityPhysics,
    pub player: Player,
}
impl GameCar {
    pub fn fixed_for_player(
        player: Player,
        position: Vec3,
        car_half_size: Vec2,
        tire_half_size: Vec2,
    ) -> GameCar {
        GameCar {
            tire_top_left: GameTire::fixed(tire_position(
                position,
                car_half_size,
                tire_half_size,
                true,
                false,
            )),
            tire_top_right: GameTire::fixed(tire_position(
                position,
                car_half_size,
                tire_half_size,
                true,
                true,
            )),
            tire_bottom_left: GameTire::fixed(tire_position(
                position,
                car_half_size,
                tire_half_size,
                false,
                false,
            )),
            tire_bottom_right: GameTire::fixed(tire_position(
                position,
                car_half_size,
                tire_half_size,
                false,
                true,
            )),
            physics: EntityPhysics::fixed(position),
            player,
        }
    }
}

#[derive(Clone, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub enum GameEntity {
    Stub(),
    Car(GameCar),
    Building(GameBuilding),
}
impl Default for GameEntity {
    fn default() -> Self {
        GameEntity::Stub()
    }
}

#[derive(Clone, Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameState {
    pub entities: Vec<GameEntity>,
}
