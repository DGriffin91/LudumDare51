use std::time::Duration;

use bevy_kira_audio::{AudioControl, AudioInstance, AudioPlugin, AudioSettings, AudioTween};
use rand::seq::SliceRandom;

use crate::{assets::AudioAssets, ui::Preferences, GameRng, GameState};

use bevy::prelude::*;
use iyes_loopless::prelude::*;

#[derive(Resource, Default)]
pub struct ConLaserAudioHandle(pub Option<Handle<AudioInstance>>);

#[derive(Resource, Default)]
pub struct MusicAudioHandle(pub Option<Handle<AudioInstance>>);

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ConLaserAudioHandle::default())
            .insert_resource(MusicAudioHandle::default())
            .insert_resource(AudioEvents::default())
            .insert_resource(AudioSettings {
                sound_capacity: 32,
                command_capacity: 32,
            })
            .add_plugin(AudioPlugin)
            .add_enter_system(GameState::RunLevel, setup_audio)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::RunLevel)
                    .with_system(run_audio)
                    .into(),
            );
    }
}

pub const MUSIC_LEVEL_CHANGED: u8 = 1 << 0;
pub const SFX_LEVEL_CHANGED: u8 = 1 << 1;
pub const LASER_SOUND: u8 = 1 << 2;
pub const WAVE_SOUND: u8 = 1 << 3;
pub const CONTINUOUS_LASER_SOUND: u8 = 1 << 4;
pub const EXPLOSION_SOUND: u8 = 1 << 5;

const SFX_OFFSET: f64 = 0.25;
const MUSIC_OFFSET: f64 = 0.15;

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct AudioEvents(pub u8);

fn setup_audio(
    mut con_laser_h: ResMut<ConLaserAudioHandle>,
    mut music_h: ResMut<MusicAudioHandle>,
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    let inst = audio
        .play(audio_assets.con_laser.clone())
        .with_volume(0.0)
        .looped()
        .handle();
    con_laser_h.0 = Some(inst);

    let inst = audio
        .play(audio_assets.music.clone())
        .with_volume(0.15)
        .fade_in(AudioTween::linear(Duration::from_secs_f32(10.0)))
        .looped()
        .handle();
    music_h.0 = Some(inst);
}

fn run_audio(
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    con_laser_h: Res<ConLaserAudioHandle>,
    mut audio_events_res: ResMut<AudioEvents>,
    music_h: Res<MusicAudioHandle>,
    pref: Res<Preferences>,
    mut rng: ResMut<GameRng>,
) {
    let sfx_level = SFX_OFFSET * pref.sfx;
    let events = **audio_events_res;
    **audio_events_res = 0; // Reset for next frame

    if sfx_level > 0.001 {
        if events & LASER_SOUND != 0 {
            audio
                .play(
                    [
                        audio_assets.laser1.clone(),
                        audio_assets.laser2.clone(),
                        audio_assets.laser3.clone(),
                        audio_assets.laser4.clone(),
                    ]
                    .choose(&mut rng.0)
                    .unwrap()
                    .clone(),
                )
                .with_volume(sfx_level * 0.65);
        }
        if events & WAVE_SOUND != 0 {
            audio
                .play(
                    [
                        audio_assets.wave1.clone(),
                        audio_assets.wave2.clone(),
                        audio_assets.wave3.clone(),
                        audio_assets.wave4.clone(),
                    ]
                    .choose(&mut rng.0)
                    .unwrap()
                    .clone(),
                )
                .with_volume(sfx_level);
        }
        if events & EXPLOSION_SOUND != 0 {
            audio
                .play(
                    [
                        audio_assets.exp1.clone(),
                        audio_assets.exp2.clone(),
                        audio_assets.exp3.clone(),
                        audio_assets.exp4.clone(),
                        audio_assets.exp5.clone(),
                        audio_assets.exp6.clone(),
                    ]
                    .choose(&mut rng.0)
                    .unwrap()
                    .clone(),
                )
                .with_volume(sfx_level);
        }
    }

    if sfx_level > 0.001 || events & SFX_LEVEL_CHANGED != 0 {
        if let Some(con_laser_h) = &con_laser_h.0 {
            if let Some(instance) = audio_instances.get_mut(con_laser_h) {
                let mut vol = 0.0;
                if events & CONTINUOUS_LASER_SOUND != 0 {
                    vol = sfx_level;
                }
                instance.set_volume(vol, AudioTween::linear(Duration::from_secs_f32(0.1)));
            }
        }
    }

    if events & MUSIC_LEVEL_CHANGED != 0 {
        if let Some(music_h) = &music_h.0 {
            if let Some(instance) = audio_instances.get_mut(music_h) {
                instance.set_volume(
                    pref.music * MUSIC_OFFSET,
                    AudioTween::linear(Duration::from_secs_f32(0.1)),
                );
            }
        }
    }
}
