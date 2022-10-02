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
    // --- Units ---
    #[asset(path = "models/units/laser.glb#Scene0")]
    pub laser_turret: Handle<Scene>,
    #[asset(path = "models/units/shockwave.glb#Scene0")]
    pub shockwave_turret: Handle<Scene>,

    #[asset(path = "models/units/rolling_unit.glb#Scene0")]
    pub rolling_enemy: Handle<Scene>,
    #[asset(path = "models/units/rolling_unit_2.glb#Scene0")]
    pub rolling_enemy_2: Handle<Scene>,
    #[asset(path = "models/units/flying_unit.glb#Scene0")]
    pub flying_enemy: Handle<Scene>,

    #[asset(path = "models/units/base.glb#Scene0")]
    pub base: Handle<Scene>,

    // --- Projectiles ---
    #[asset(path = "models/projectiles/laser_blast.glb#Scene0")]
    pub projectile_laser_blast: Handle<Scene>,

    // --- Misc ---
    #[asset(path = "models/misc/disc.glb#Scene0")]
    pub disc: Handle<Scene>,
}
