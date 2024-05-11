use crate::domain::car_body::*;
use crate::domain::game_state::*;
use crate::domain::player::Player;
use crate::domain::spritesheets::SpriteSheets;
use crate::domain::tire::Tire;
use crate::logic::math::*;
use bevy::prelude::*;
use bevy_ggrs::AddRollbackCommandExtension;
use bevy_rapier2d::prelude::*;
use bevy_sprite3d::Sprite3dParams;

pub fn spawn_car(
    spritesheets: &SpriteSheets,
    sprite_params: &mut Sprite3dParams,
    pixels_per_meter: f32,
    commands: &mut Commands,
    spawn_pool: &mut Vec<Entity>,
    player: Player,
    car_title: String,
    car_half_size: Vec2,
    car_radius: f32,
    tire_half_size: Vec2,
    car_color: Color,
    tire_color: Color,
    tire_damping: Damping,
    car_physics: GameCar,
) {
    let mut car = commands.entity(spawn_pool.pop().unwrap());
    car.insert(CarBody::build(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        car_title.clone(),
        car_half_size,
        car_radius,
        player.clone(),
        car_color,
        car_physics.physics.clone(),
    ));
    car.add_rollback();

    let car_id = car.id();
    spawn_tires(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        commands,
        spawn_pool,
        player.clone(),
        car_id.clone(),
        car_half_size,
        tire_half_size,
        tire_color,
        tire_damping,
        car_title,
        car_physics,
    );
}

fn spawn_tire(
    spritesheets: &SpriteSheets,
    sprite_params: &mut Sprite3dParams,
    pixels_per_meter: f32,
    commands: &mut Commands,
    spawn_pool: &mut Vec<Entity>,
    player: Player,
    car: Entity,
    car_anchor: impl Into<GenericJoint>,
    tire_half_size: Vec2,
    is_front: bool,
    is_right: bool,
    car_title: String,
    color: Color,
    damping: Damping,
    physics: GameTire,
) {
    let mut tire = commands.entity(spawn_pool.pop().unwrap());
    tire.insert(Tire::build(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        player,
        is_front,
        is_right,
        tire_half_size,
        car_title,
        color,
        damping,
        physics,
    ));
    tire.add_rollback();
    tire.insert(ImpulseJoint::new(car, car_anchor));
}

pub fn spawn_tires(
    spritesheets: &SpriteSheets,
    sprite_params: &mut Sprite3dParams,
    pixels_per_meter: f32,
    commands: &mut Commands,
    spawn_pool: &mut Vec<Entity>,
    player: Player,
    car: Entity,
    car_half_size: Vec2,
    tire_half_size: Vec2,
    color: Color,
    damping: Damping,
    car_title: String,
    car_physics: GameCar,
) {
    spawn_tire(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        commands,
        spawn_pool,
        player.clone(),
        car,
        tire_anchor(car_half_size, tire_half_size, true, true),
        tire_half_size,
        true,
        true,
        car_title.clone(),
        color,
        damping,
        car_physics.tire_top_right,
    );
    spawn_tire(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        commands,
        spawn_pool,
        player.clone(),
        car,
        tire_anchor(car_half_size, tire_half_size, true, false),
        tire_half_size,
        true,
        false,
        car_title.clone(),
        color,
        damping,
        car_physics.tire_top_left,
    );
    spawn_tire(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        commands,
        spawn_pool,
        player.clone(),
        car,
        tire_anchor(car_half_size, tire_half_size, false, true),
        tire_half_size,
        false,
        true,
        car_title.clone(),
        color,
        damping,
        car_physics.tire_bottom_right,
    );
    spawn_tire(
        spritesheets,
        sprite_params,
        pixels_per_meter,
        commands,
        spawn_pool,
        player.clone(),
        car,
        tire_anchor(car_half_size, tire_half_size, false, false),
        tire_half_size,
        false,
        false,
        car_title.clone(),
        color,
        damping,
        car_physics.tire_bottom_left,
    );
}

pub fn tire_anchor(
    car_half_size: Vec2,
    tire_half_size: Vec2,
    is_front: bool,
    is_right: bool,
) -> impl Into<GenericJoint> {
    let car_anchor = Vec2 {
        x: signed(is_right, car_half_size.x + tire_half_size.x * 0.5),
        y: signed(is_front, car_half_size.y - tire_half_size.y * 2.3),
    };
    let tire_anchor = Vec2::new(0.0, 0.0);

    RevoluteJointBuilder::new()
        .local_anchor1(car_anchor)
        .local_anchor2(tire_anchor)
}

pub fn tire_position(
    car_position: Vec3,
    car_half_size: Vec2,
    tire_half_size: Vec2,
    is_front: bool,
    is_right: bool,
) -> Vec3 {
    Vec3 {
        x: car_position.x + signed(is_right, car_half_size.x + tire_half_size.x * 0.5),
        y: car_position.y + signed(is_front, car_half_size.y - tire_half_size.y * 2.3),
        z: 0.0,
    }
}
