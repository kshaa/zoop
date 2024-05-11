use crate::domain::room_config::RoomConfig;
use crate::logic::math::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use url::{ParseError, Url};
use zoop_shared::network_player::NetworkPlayer;

#[derive(Resource, Clone, Debug)]
pub struct GameConfig {
    // Native Bevy units:
    // - distance in pixels
    // - angles in radians
    // - percentages from 0.0 to 1.0
    pub network: Option<RoomConfig>,
    pub players: Vec<NetworkPlayer>,
    pub fps: u16,
    pub load_seconds: u16,
    pub canvas_selector: Option<String>,
    pub pixels_per_meter: f32,
    pub default_camera_height: f32,
    pub parking_camera_height: f32,
    pub car_half_width: f32,
    pub car_half_length: f32,
    pub car_radius: f32,
    pub tire_half_thickness: f32,
    pub tire_radius: f32,
    pub tire_rotation_velocity_reduction_coefficient: f32,
    pub tire_rotation_step_reduction_coefficient: f32,
    pub tire_rotation_per_tick: f32,
    pub tire_min_angle: f32,
    pub tire_max_angle: f32,
    pub parking_force: f32,
    pub tire_acceleration_force: f32,
    pub tire_reversing_force: f32,
    pub tire_breaking_force: f32,
    pub tire_friction_force: f32,
    pub tire_linear_damping: f32,
    pub tire_angular_damping: f32,
    pub drift_velocity: f32,
    pub drift_loss_per_tick: f32,
    pub camera_velocity_coefficient: f32,
    pub desync_max_frames: u16,
}

const FPS: u16 = 60;
pub const DESYNC_MAX_FRAMES: u16 = 30;

impl GameConfig {
    fn _meters2pix(pixels_per_meter: f32, meters: f32) -> f32 {
        pixels_per_meter * meters
    }
    pub fn meters2pix(&self, meters: f32) -> f32 {
        self.pixels_per_meter * meters
    }

    pub fn default(
        network: Option<RoomConfig>,
        players: Vec<NetworkPlayer>,
        canvas_selector: Option<String>,
    ) -> GameConfig {
        let ppm = 10.0;
        let m2p = |meters: f32| GameConfig::_meters2pix(ppm, meters);
        GameConfig {
            network,
            players,
            fps: FPS,
            load_seconds: 1,
            canvas_selector,
            pixels_per_meter: ppm,
            default_camera_height: m2p(80.0),
            parking_camera_height: m2p(30.0),
            car_half_width: m2p(0.65),
            car_half_length: m2p(1.8),
            car_radius: 0.15,
            tire_half_thickness: m2p(0.2),
            tire_radius: m2p(0.4),
            tire_rotation_velocity_reduction_coefficient: 0.01,
            tire_rotation_step_reduction_coefficient: 0.5,
            tire_rotation_per_tick: deg2rad(1.0),
            tire_min_angle: deg2rad(15.0),
            tire_max_angle: deg2rad(35.0),
            parking_force: 0.1,
            tire_acceleration_force: m2p(40.0),
            tire_reversing_force: m2p(30.0),
            tire_breaking_force: m2p(30.0),
            tire_friction_force: 0.9,
            tire_linear_damping: 5.0,
            tire_angular_damping: 0.1,
            drift_velocity: m2p(10.0),
            drift_loss_per_tick: 0.5 * (1.0 / (FPS as f32)),
            camera_velocity_coefficient: 0.04,
            desync_max_frames: DESYNC_MAX_FRAMES,
        }
    }

    pub fn game_room_address(&self) -> Option<Result<Url, ParseError>> {
        self.network.clone().map(|network| {
            network.server_address.join(
                format!(
                    "/api/game/connect/{}/as/{}/ticket/{}",
                    network.room.0.to_string(),
                    network.user_id.0.to_string(),
                    network.user_ticket
                )
                .as_str(),
            )
        })
    }

    pub fn tire_damping(&self) -> Damping {
        Damping {
            linear_damping: self.tire_linear_damping,
            angular_damping: self.tire_angular_damping,
        }
    }

    pub fn tire_half_size(&self) -> Vec2 {
        Vec2 {
            x: self.tire_half_thickness,
            y: self.tire_radius,
        }
    }

    pub fn car_half_size(&self) -> Vec2 {
        Vec2 {
            x: self.car_half_width,
            y: self.car_half_length,
        }
    }

    pub fn rapier_config(&self) -> RapierConfiguration {
        RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: 1.0 / f32::from(self.fps),
                substeps: 1,
            },
            gravity: Vec2::ZERO,
            query_pipeline_active: false,
            physics_pipeline_active: false,
            force_update_from_transform_changes: true,
            ..RapierConfiguration::new(self.pixels_per_meter)
        }
    }
}
