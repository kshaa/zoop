use crate::domain::spritesheets::SpriteSheets;
use bevy::math::*;
use bevy::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dParams};

#[derive(Bundle)]
pub struct Trace {
    sprite3d: Sprite3dBundle
}

impl Trace {
    pub fn build(
        spritesheets: &SpriteSheets,
        sprite_params: &mut Sprite3dParams,
        pixels_per_meter: f32,
        transform: Transform,
    ) -> Trace {
        Trace {
            sprite3d: Sprite3d {
                image: spritesheets.trace.clone(),
                pixels_per_metre: 500.0 / pixels_per_meter,
                // custom_size: Some(half_size * 2.0),
                // partial_alpha: true,
                unlit: true,
                transform,
                ..default()
            }.bundle(sprite_params),
        }
    }
}