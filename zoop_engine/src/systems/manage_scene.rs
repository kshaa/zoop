use crate::domain::car::spawn_car;
use bevy::asset::LoadState;
use bevy::gltf::{Gltf, GltfMesh};


use crate::domain::colors::{ZOOP_BLACK, ZOOP_RED, ZOOP_YELLOW};
use crate::domain::game_config::GameConfig;
use crate::domain::game_state::{GameBuilding, GameCar, GameEntity, GameState};
use crate::domain::player::Player;
use crate::domain::spawn::DeterministicSpawn;

use crate::domain::building::Building;
use crate::domain::game_readiness::GameReadiness;
use crate::domain::spritesheets::SpriteSheets;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_ggrs::{Rollback, RollbackIdProvider};
use bevy_rapier2d::prelude::*;
use bevy_sprite3d::Sprite3dParams;

pub fn init_scene(config: &GameConfig) -> GameState {
    info!("Initiating scene state");

    let building_half_size = config.meters2pix(10.0);

    let mut cars = config
        .players
        .iter()
        .enumerate()
        .map(|(handle, _)| {
            let player = Player { handle };
            let position = Vec3 {
                x: building_half_size + config.car_half_size().x * 6.0 * (handle as f32),
                y: 0.0,
                z: 0.0,
            };
            let tire_half_size = config.tire_half_size();
            let car_half_size = config.car_half_size();

            GameEntity::Car(GameCar::fixed_for_player(
                player,
                position,
                car_half_size,
                tire_half_size,
            ))
        })
        .collect();

    let buildings: Vec<(i32, i32, u32, bool)> = vec!(
                           (0,10, 2, false), (1,10, 1, false),(2,10, 2, false),
                           (0, 9, 2, true),                   (2, 9, 1, false),
                           (0, 8, 2, false),                  (2, 8, 2, false),
        (-1, 7, 2, false), (0, 7, 2, false), (1, 7, 2, true), (2, 7, 2, false), (3, 7, 2, false), (4, 7, 2, false),
        (-1, 6, 2, false),
        (-1, 5, 2, false),                   (1, 5, 2, false), (2, 5, 1, false), (3, 5, 2, false),
        (-1, 4, 2, false),                   (1, 4, 1, false),
        (-1, 3, 1, false),                   (1, 3, 2, false),
        (-1, 2, 2, false),                   (1, 2, 1, false),
        (-1, 1, 1, false),                   (1, 1, 2, false),
        (-1, 0, 2, false),                   (1, 0, 1, false),
    );
    fn make_building(building_half_size: f32, x: f32, y: f32, stories: u32, is_tunnel: bool) -> GameEntity {
        GameEntity::Building(GameBuilding::of(
            Vec3 {
                x: building_half_size * 2.0 * x,
                y: building_half_size * 2.0 * y,
                z: building_half_size,
            },
            stories,
            is_tunnel,
            building_half_size,
        ))
    }

    let mut buildings =
        buildings
            .iter()
            .map(|(x, y, stories, is_tunnel)|
                make_building(
                    building_half_size,
                    x.clone() as f32,
                    y.clone() as f32,
                    stories.clone(),
                    is_tunnel.clone()))
            .collect();

    let mut entities = vec![];
    entities.append(&mut cars);
    entities.append(&mut buildings);

    GameState { entities }
}

#[allow(dead_code)]
pub fn destroy_scene(
    config: Res<GameConfig>,
    player_entities: Query<(Entity, With<Player>)>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut rapier_render_time: ResMut<SimulationToRenderTime>,
    mut rapier_context: ResMut<RapierContext>,
    mut rapier_collision_events: ResMut<Events<CollisionEvent>>,
    mut rapier_force_events: ResMut<Events<ContactForceEvent>>,
    mut commands: Commands,
) {
    info!("Destroying rapier context");
    for (entity, _) in player_entities.iter() {
        commands.entity(entity).despawn();
    }
    *rapier_config = config.rapier_config();
    *rapier_render_time = SimulationToRenderTime::default();
    *rapier_collision_events = Events::<CollisionEvent>::default();
    *rapier_force_events = Events::<ContactForceEvent>::default();
    *rapier_context = RapierContext::default();
}

pub fn init_materials(
    mut spritesheets: ResMut<SpriteSheets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    let cube = meshes.add(shape::Cube::default().into());
    spritesheets.debug_material = debug_material;
    spritesheets.cube = cube;
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

pub fn await_assets(
    asset_server: Res<AssetServer>,
    mut background: ResMut<ClearColor>,
    mut next_state: ResMut<NextState<GameReadiness>>,
    spritesheets: Res<SpriteSheets>,
) {
    // Check assets loaded
    if asset_server.get_load_state(&spritesheets.car) != LoadState::Loaded {
        return;
    }
    if asset_server.get_load_state(&spritesheets.tire) != LoadState::Loaded {
        return;
    }
    if asset_server.get_load_state(&spritesheets.trace) != LoadState::Loaded {
        return;
    }
    if asset_server.get_load_state(&spritesheets.building) != LoadState::Loaded {
        return;
    }

    next_state.set(GameReadiness::LoadingScene);
}

pub fn setup_scene(
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut background: ResMut<ClearColor>,
    mut next_state: ResMut<NextState<GameReadiness>>,
    spritesheets: Res<SpriteSheets>,
    mut sprite_params: Sprite3dParams,
    config: Res<GameConfig>,
    state: Res<GameState>,
    mut rip: ResMut<RollbackIdProvider>,
    spawn_pool: Query<(Entity, &DeterministicSpawn)>,
    mut commands: Commands,
) {
    // Visual loading state logic
    next_state.set(GameReadiness::Ready);
    background.0 = ZOOP_YELLOW;

    // Get our entities and sort them by the spawn component index
    let mut sorted_spawn_pool: Vec<(Entity, &DeterministicSpawn)> = spawn_pool.iter().collect();
    sorted_spawn_pool.sort_by_key(|e| e.1.index);
    // Get the Entities in reverse for easy popping
    let mut sorted_entity_pool: Vec<Entity> = sorted_spawn_pool.iter().map(|p| p.0).rev().collect();

    spawn_scene(
        &spritesheets,
        &gltfs,
        &gltf_meshes,
        &mut sprite_params,
        config.as_ref(),
        &state,
        &mut commands,
        &mut sorted_entity_pool,
        &mut rip,
    );
}

pub fn spawn_scene(
    spritesheets: &SpriteSheets,
    gltfs: &Assets<Gltf>,
    gltf_meshes: &Assets<GltfMesh>,
    sprite_params: &mut Sprite3dParams,
    config: &GameConfig,
    state: &GameState,
    commands: &mut Commands,
    spawn_pool: &mut Vec<Entity>,
    rip: &mut RollbackIdProvider,
) {
    info!("Spawning scene from state");
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    for entity in state.entities.iter() {
        match entity {
            GameEntity::Stub() => (),
            GameEntity::Car(car) => {
                info!("Spawning car for player {}", car.player.handle);
                setup_car(
                    spritesheets,
                    sprite_params,
                    config,
                    car.clone(),
                    commands,
                    spawn_pool,
                    rip,
                )
            }
            GameEntity::Building(building) => setup_building(
                spritesheets,
                gltfs,
                gltf_meshes,
                config,
                building.clone(),
                commands,
                spawn_pool,
                rip,
            ),
        }
    }

    while !spawn_pool.is_empty() {
        let mut leftover = commands.entity(spawn_pool.pop().unwrap());
        leftover.despawn();
    }
}

pub fn setup_building(
    spritesheets: &SpriteSheets,
    gltfs: &Assets<Gltf>,
    gltf_meshes: &Assets<GltfMesh>,
    _config: &GameConfig,
    building: GameBuilding,
    commands: &mut Commands,
    spawn_pool: &mut Vec<Entity>,
    rip: &mut RollbackIdProvider,
) {
    if !building.is_tunnel {
        let mut physics_entity = commands.entity(spawn_pool.pop().unwrap());
        physics_entity.insert(Rollback::new(rip.next_id()));
        physics_entity.insert(Building::build(&building));
    }

    let start = if building.is_tunnel { 1 } else { 0 };
    for story in start..building.stories {
        let mut mesh_entity = commands.entity(spawn_pool.pop().unwrap());
        mesh_entity.insert(Building::build_mesh(spritesheets, gltfs, gltf_meshes, &building, story));
    }

}

pub fn setup_car(
    spritesheets: &SpriteSheets,
    sprite_params: &mut Sprite3dParams,
    config: &GameConfig,
    car: GameCar,
    commands: &mut Commands,
    spawn_pool: &mut Vec<Entity>,
    rip: &mut RollbackIdProvider,
) {
    spawn_car(
        spritesheets,
        sprite_params,
        config.pixels_per_meter,
        commands,
        spawn_pool,
        rip,
        car.player.clone(),
        String::from(format!("Car #{}", car.player.handle)),
        config.car_half_size(),
        config.car_radius,
        config.tire_half_size(),
        ZOOP_RED,
        ZOOP_BLACK,
        config.tire_damping(),
        car,
    );
}
