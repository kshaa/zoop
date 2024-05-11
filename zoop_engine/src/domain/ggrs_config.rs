use crate::domain::controls::Controls;
use bevy_ggrs::ggrs::Config;
use zoop_shared::player_id::PlayerId;

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = Controls;
    // Docs say this can be left as u8 :shrugs:
    type State = u8;
    type Address = PlayerId;
}
