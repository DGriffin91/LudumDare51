use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

#[derive(Resource, AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/ShareTechMono-Regular.ttf")]
    pub mono_medium: Handle<Font>,
}

#[derive(Resource, AssetCollection)]
pub struct ModelAssets {
    // --- Units ---
    #[asset(path = "models/units/laser.glb#Scene0")]
    pub blaster_turret: Handle<Scene>,
    #[asset(path = "models/units/shockwave.glb#Scene0")]
    pub wave_turret: Handle<Scene>,
    #[asset(path = "models/units/laser_turret_2.glb#Scene0")]
    pub laser_turret: Handle<Scene>,

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

#[derive(Resource, AssetCollection)]
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

    #[asset(path = "audio/music/music.ogg")]
    pub music: Handle<AudioSource>,
}

/*
TODO game in 0.9 looks different than in 0.8
There was an issue with a clipping plane in board.glb that has been fixed, but it still looks different.
Things like the 2nd ground unit looked more orange before.
https://github.com/bevyengine/bevy/pull/6828 will mess with this again with 0.10
The fix below isn't the right one, but may be a starting place to find the right fix.

#[inline]
fn linear_to_nonlinear_srgb(x: f32) -> f32 {
    if x <= 0.0 {
        return x;
    }

    if x <= 0.0031308 {
        x * 12.92 // linear falloff in dark values
    } else {
        (1.055 * x.powf(1.0 / 2.4)) - 0.055 // gamma curve in other area
    }
}


#[inline]
fn nonlinear_to_linear_srgb(x: f32) -> f32 {
    if x <= 0.0 {
        return x;
    }
    if x <= 0.04045 {
        x / 12.92 // linear falloff in dark values
    } else {
        ((x + 0.055) / 1.055).powf(2.4) // gamma curve in other area
    }
}

pub fn fix_material_colors(
    mut events: EventReader<AssetEvent<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut mat) = materials.get_mut(handle) {
                    dbg!(mat.base_color);
                    let c: Vec4 = mat.base_color.into();
                    mat.base_color = Color::rgba(
                        nonlinear_to_linear_srgb(c.x),
                        nonlinear_to_linear_srgb(c.y),
                        nonlinear_to_linear_srgb(c.z),
                        nonlinear_to_linear_srgb(c.w),
                    );
                    dbg!(mat.base_color);
                    let c: Vec4 = mat.emissive.into();
                    mat.emissive = Color::rgba(
                        nonlinear_to_linear_srgb(c.x),
                        nonlinear_to_linear_srgb(c.y),
                        nonlinear_to_linear_srgb(c.z),
                        nonlinear_to_linear_srgb(c.w),
                    );
                }
            }
            AssetEvent::Modified { .. } => (),
            AssetEvent::Removed { .. } => (),
        }
    }
}

*/
