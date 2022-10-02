use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    RunLevel,
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraMono-Medium.ttf")]
    pub fira_mono_medium: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct ModelAssets {
    #[asset(path = "models/units/laser.glb#Scene0")]
    pub laser_turret: Handle<Scene>,
    #[asset(path = "models/units/shockwave.glb#Scene0")]
    pub shockwave_turret: Handle<Scene>,
    #[asset(path = "models/projectiles/laser_blast.glb#Scene0")]
    pub projectile_laser_blast: Handle<Scene>,
    #[asset(path = "models/misc/disc.glb#Scene0")]
    pub disc: Handle<Scene>,
}
