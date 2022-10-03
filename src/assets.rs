use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    RunLevel,
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/ShareTechMono-Regular.ttf")]
    pub mono_medium: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct ModelAssets {
    // --- Units ---
    #[asset(path = "models/units/laser.glb#Scene0")]
    pub laser_turret: Handle<Scene>,
    #[asset(path = "models/units/shockwave.glb#Scene0")]
    pub shockwave_turret: Handle<Scene>,
    #[asset(path = "models/units/laser_turret_2.glb#Scene0")]
    pub laser_turret_2: Handle<Scene>,

    #[asset(path = "models/units/rolling_unit.glb#Scene0")]
    pub rolling_enemy: Handle<Scene>,
    #[asset(path = "models/units/rolling_unit_2.glb#Scene0")]
    pub rolling_enemy_2: Handle<Scene>,
    #[asset(path = "models/units/flying_unit.glb#Scene0")]
    pub flying_enemy: Handle<Scene>,

    #[asset(path = "models/units/base.glb#Scene0")]
    pub base: Handle<Scene>,
    #[asset(path = "models/units/base_destroyed.glb#Scene0")]
    pub base_destroyed: Handle<Scene>,

    // --- Projectiles ---
    #[asset(path = "models/projectiles/laser_blast.glb#Scene0")]
    pub projectile_laser_blast: Handle<Scene>,

    // --- Misc ---
    #[asset(path = "models/misc/disc.glb#Scene0")]
    pub disc: Handle<Scene>,
    #[asset(path = "models/misc/board.glb#Scene0")]
    pub board: Handle<Scene>,
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    // --- Units ---
    #[asset(path = "audio/units/laser1.flac")]
    pub laser1: Handle<AudioSource>,
    #[asset(path = "audio/units/laser2.flac")]
    pub laser2: Handle<AudioSource>,
    #[asset(path = "audio/units/laser3.flac")]
    pub laser3: Handle<AudioSource>,
    #[asset(path = "audio/units/laser4.flac")]
    pub laser4: Handle<AudioSource>,

    #[asset(path = "audio/units/wave1.flac")]
    pub wave1: Handle<AudioSource>,
    #[asset(path = "audio/units/wave2.flac")]
    pub wave2: Handle<AudioSource>,
    #[asset(path = "audio/units/wave3.flac")]
    pub wave3: Handle<AudioSource>,
    #[asset(path = "audio/units/wave4.flac")]
    pub wave4: Handle<AudioSource>,

    #[asset(path = "audio/units/exp1.flac")]
    pub exp1: Handle<AudioSource>,
    #[asset(path = "audio/units/exp2.flac")]
    pub exp2: Handle<AudioSource>,
    #[asset(path = "audio/units/exp3.flac")]
    pub exp3: Handle<AudioSource>,
    #[asset(path = "audio/units/exp4.flac")]
    pub exp4: Handle<AudioSource>,
    #[asset(path = "audio/units/exp5.flac")]
    pub exp5: Handle<AudioSource>,
    #[asset(path = "audio/units/exp6.flac")]
    pub exp6: Handle<AudioSource>,

    #[asset(path = "audio/units/con_laser.flac")]
    pub con_laser: Handle<AudioSource>,
}
