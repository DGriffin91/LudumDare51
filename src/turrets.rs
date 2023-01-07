use std::f32::INFINITY;
use std::time::Duration;

use bevy::math::*;
use bevy::prelude::*;
use bevy_scene_hook::HookedSceneBundle;
use bevy_scene_hook::SceneHook;

use crate::audio::AudioEvents;
use crate::audio::CONTINUOUS_LASER_SOUND;
use crate::audio::LASER_SOUND;
use crate::audio::WAVE_SOUND;
use crate::basic_light;
use crate::player::PlayerState;
use crate::schedule::TIMESTEP;
use crate::schedule::TIMESTEP_MILLI;
use crate::ui::Preferences;

use crate::{
    assets::ModelAssets,
    enemies::{Enemy, Health},
};

#[derive(Clone, Copy, Component, PartialEq, Eq)]
pub enum Turret {
    Blaster,
    Laser,
    Wave,
}

impl Turret {
    pub fn cost(&self) -> u64 {
        match self {
            Turret::Blaster => 100,
            Turret::Wave => 200,
            Turret::Laser => 300,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AttackDamage(pub f32);

#[derive(Component, Deref, DerefMut)]
pub struct Cooldown(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct Range(f32);

#[derive(Component)]
pub struct ContinuousLaserLight;

impl Turret {
    pub fn spawn_shockwave_turret(
        com: &mut Commands,
        trans: Vec3,
        model_assets: &ModelAssets,
        pref: &Preferences,
    ) -> (Turret, bevy::prelude::Entity) {
        let mut ecmds = com.spawn_empty();
        let entity_id = ecmds.id();

        ecmds
            .insert(AttackDamage(0.1))
            .insert(Cooldown(Timer::new(
                Duration::from_secs_f32(0.9),
                TimerMode::Repeating,
            )))
            .insert(Range(4.0))
            .insert(Turret::Wave);
        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.2, 1.0),
            200.0,
            1.8 * pref.light_r,
            0.5,
            vec3(0.0, 0.6, 0.0),
        );

        ecmds.insert(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.wave_turret.clone(),
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |entity, cmds| {
                if let Some(name) = entity.get::<Name>() {
                    if name.contains("BobbleSphere") {
                        cmds.insert(ShockwaveSphere {
                            top_parent: entity_id,
                            phase: 0.0,
                        });
                    }
                    if name.contains("Top Cap") {
                        cmds.insert(Cap {
                            top_parent: entity_id,
                            progress: 0.0,
                            direction: Vec3::Y * 0.15,
                        });
                    }
                    if name.contains("Bottom Cap") {
                        cmds.insert(Cap {
                            top_parent: entity_id,
                            progress: 0.0,
                            direction: -Vec3::Y * 0.15,
                        });
                    }
                }
            }),
        });

        (Turret::Wave, entity_id)
    }

    pub fn spawn_blaster_turret(
        com: &mut Commands,
        trans: Vec3,
        model_assets: &ModelAssets,
        pref: &Preferences,
    ) -> (Turret, bevy::prelude::Entity) {
        let mut ecmds = com.spawn_empty();
        let entity_id = ecmds.id();
        ecmds
            .insert(AttackDamage(0.02))
            .insert(Cooldown(Timer::new(
                Duration::from_secs_f32(0.5),
                TimerMode::Repeating,
            )))
            .insert(Range(10.0))
            .insert(Turret::Blaster);
        basic_light(
            &mut ecmds,
            Color::rgb(0.3, 0.0, 1.0),
            110.0,
            1.8 * (pref.light_r + 0.2),
            0.5,
            vec3(0.0, 1.0, 0.0),
        );

        ecmds.insert(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.blaster_turret.clone(),
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |entity, cmds| {
                if let Some(name) = entity.get::<Name>() {
                    if name.contains("Head") {
                        cmds.insert(Swivel {
                            top_parent: entity_id,
                        });
                    }
                }
            }),
        });

        (Turret::Blaster, entity_id)
    }

    pub fn spawn_laser_continuous_turret(
        com: &mut Commands,
        trans: Vec3,
        model_assets: &ModelAssets,
        pref: &Preferences,
    ) -> (Turret, bevy::prelude::Entity) {
        let mut ecmds = com.spawn_empty();
        let entity_id = ecmds.id();
        ecmds
            .insert(AttackDamage(0.35))
            .insert(Cooldown(Timer::new(
                Duration::from_secs_f32(0.1),
                TimerMode::Once,
            )))
            .insert(Range(16.0))
            .insert(Turret::Laser);

        ecmds.add_children(|parent| {
            parent
                .spawn(PointLightBundle {
                    point_light: PointLight {
                        color: Color::hex("68FF72").unwrap(),
                        intensity: 110.0,
                        range: 2.2 * pref.light_r,
                        radius: 0.5,
                        ..default()
                    },
                    transform: Transform::from_translation(vec3(0.0, 1.0, 0.0)),
                    ..default()
                })
                .insert(ContinuousLaserLight);
        });

        ecmds.insert(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.laser_turret.clone(),
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |entity, cmds| {
                if let Some(name) = entity.get::<Name>() {
                    if name.contains("Diamond Lasers") {
                        cmds.insert(DiamondLasers {
                            top_parent: entity_id,
                        });
                    }
                    if name.contains("Laser Beam") {
                        cmds.insert(LaserBeam {
                            top_parent: entity_id,
                        });
                    }
                }
            }),
        });

        (Turret::Blaster, entity_id)
    }
}

pub fn reset_turret_gfx(
    mut diamond_lasers: Query<&mut Visibility, (With<DiamondLasers>, Without<LaserBeam>)>,
    mut laser_beams: Query<&mut Visibility, (With<LaserBeam>, Without<DiamondLasers>)>,
    mut continuous_laser_light: Query<&mut PointLight, With<ContinuousLaserLight>>,
) {
    for mut vis in diamond_lasers.iter_mut() {
        vis.is_visible = false;
    }
    for mut vis in laser_beams.iter_mut() {
        vis.is_visible = false;
    }
    for mut light in continuous_laser_light.iter_mut() {
        light.intensity = 90.0;
    }
}

pub fn turret_fire(
    mut com: Commands,

    mut turrets: Query<
        (
            Entity,
            &Transform,
            &AttackDamage,
            &Range,
            &mut Cooldown,
            &Turret,
        ),
        (
            Without<LaserBeam>,
            Without<DiamondLasers>,
            Without<Disabled>,
        ),
    >,
    mut enemies: Query<
        (Entity, &Transform, &mut Health),
        (With<Enemy>, Without<LaserBeam>, Without<DiamondLasers>),
    >,
    model_assets: Res<ModelAssets>,
    mut caps: Query<&mut Cap>,
    mut diamond_lasers: Query<
        (&mut Visibility, &DiamondLasers),
        (With<DiamondLasers>, Without<LaserBeam>),
    >,
    mut laser_beams: Query<
        (&mut Transform, &mut Visibility, &LaserBeam),
        (With<LaserBeam>, Without<DiamondLasers>),
    >,
    mut continuous_laser_light: Query<(&Parent, &mut PointLight), With<ContinuousLaserLight>>,
    player: Res<PlayerState>,
    pref: Res<Preferences>,
    mut audio_events: ResMut<AudioEvents>,
) {
    if !player.alive() {
        return;
    }

    for (turret_entity, turret_trans, damage, range, mut cooldown, turret) in turrets.iter_mut() {
        cooldown.tick(Duration::from_millis(TIMESTEP_MILLI));

        let mut closest = None;
        let mut closest_dist = INFINITY;

        for (entity, enemy_trans, _health) in enemies.iter_mut() {
            let dist = turret_trans.translation.distance(enemy_trans.translation);
            if dist < closest_dist {
                closest = Some(entity);
                closest_dist = dist;
            }
        }

        if cooldown.finished() {
            match turret {
                Turret::Blaster => {
                    if let Some(entity) = closest {
                        if let Ok((entity, enemy_trans, mut _health)) = enemies.get_mut(entity) {
                            let dist = enemy_trans.translation.distance(turret_trans.translation);
                            if dist < **range {
                                cooldown.reset();
                                let turret_head_trans = turret_trans.translation + Vec3::Y * 1.0;
                                let fire_dir =
                                    (enemy_trans.translation - turret_head_trans).normalize();
                                let mut ecmds = com.spawn_empty();

                                ecmds.insert(SceneBundle {
                                    scene: model_assets.projectile_laser_blast.clone(),
                                    transform: Transform::from_translation(turret_head_trans)
                                        .looking_at(enemy_trans.translation, Vec3::Y),
                                    ..default()
                                });
                                ecmds.insert(Projectile {
                                    dir: fire_dir,
                                    speed: 0.5,
                                    dest: enemy_trans.translation,
                                    enemy: entity,
                                    damage: damage.0,
                                    blast_radius: 1.5,
                                    hit: false,
                                    hit_despawn_countdown: 1.0,
                                });
                                if !pref.less_lights {
                                    basic_light(
                                        &mut ecmds,
                                        Color::rgb(0.2, 0.0, 1.0),
                                        100.0,
                                        1.5,
                                        1.0,
                                        Vec3::Y * -0.5,
                                    );
                                }
                                **audio_events |= LASER_SOUND;
                            }
                        }
                    }
                }
                Turret::Laser => {
                    if let Some(entity) = closest {
                        if let Ok((_entity, enemy_trans, mut health)) = enemies.get_mut(entity) {
                            if closest_dist < **range {
                                //cooldown.reset(); Don't ever reset continuous
                                health.0 -= damage.0 * TIMESTEP * player.laser_upgrade;
                                for (mut vis, laser) in diamond_lasers.iter_mut() {
                                    if laser.top_parent == turret_entity {
                                        vis.is_visible = true;
                                        **audio_events |= CONTINUOUS_LASER_SOUND;
                                    }
                                }
                                for (mut trans, mut vis, laser) in laser_beams.iter_mut() {
                                    if laser.top_parent == turret_entity {
                                        vis.is_visible = true;
                                        let dir = (enemy_trans.translation
                                            - turret_trans.translation
                                            - Vec3::Y * 1.0)
                                            .normalize();
                                        let t = trans.translation + dir;
                                        trans.look_at(t, Vec3::Y);
                                    }
                                }

                                for (parent, mut light) in continuous_laser_light.iter_mut() {
                                    if parent.get() == turret_entity {
                                        light.intensity = 130.0;
                                    }
                                }
                            }
                        }
                    }
                }
                Turret::Wave => {
                    for (_entity, enemy_trans, mut health) in enemies.iter_mut() {
                        let dist = enemy_trans.translation.distance(turret_trans.translation);
                        if dist < **range {
                            for mut cap in caps.iter_mut() {
                                if cap.top_parent == turret_entity {
                                    cap.progress = 1.0;
                                }
                            }
                            cooldown.reset();
                            health.0 -= damage.0 * (1.0 / dist.max(1.0)) * player.wave_upgrade;
                            let mut ecmds = com.spawn(SceneBundle {
                                scene: model_assets.disc.clone(),
                                transform: Transform::from_translation(
                                    turret_trans.translation + Vec3::Y * 0.5,
                                ),
                                ..Default::default()
                            });
                            ecmds.insert(DiscExplosion {
                                speed: 9.0,
                                size: 4.0,
                                progress: 0.0,
                            });
                            if !pref.less_lights {
                                basic_light(
                                    &mut ecmds,
                                    Color::rgb(1.0, 0.0, 1.0),
                                    70.0,
                                    3.5,
                                    1.0,
                                    Vec3::Y,
                                );
                            }
                            **audio_events |= WAVE_SOUND;
                        }
                    }
                }
            }
        }
    }
}

pub fn blaster_point_at_enemy(
    mut turrets: Query<
        (Entity, &mut Transform, &Range),
        (With<Turret>, Without<Swivel>, Without<Disabled>),
    >,
    mut swivels: Query<(&mut Transform, &Swivel), Without<Turret>>,
    mut enemies: Query<&Transform, (With<Enemy>, (Without<Turret>, Without<Swivel>))>,
    player: Res<PlayerState>,
) {
    if !player.alive() {
        return;
    }
    for (turret_entity, turret_trans, _range) in turrets.iter_mut() {
        let mut closest = Vec3::ZERO;
        let mut closest_dist = INFINITY;
        for enemy_trans in enemies.iter_mut() {
            let dist = turret_trans.translation.distance(enemy_trans.translation);
            if dist < closest_dist {
                closest = enemy_trans.translation;
                closest_dist = dist;
            }
        }
        let dir = (closest - turret_trans.translation - Vec3::Y * 1.0).normalize();
        for (mut swivel_trans, swivel) in swivels.iter_mut() {
            if swivel.top_parent == turret_entity {
                //let before = swivel_trans.rotation.to_euler(EulerRot::XYZ);
                //let after = swivel_trans
                //    .looking_at(closest, Vec3::Y)
                //    .rotation
                //    .to_euler(EulerRot::XYZ);

                // TODO doesn't work if the parent turret has rotation
                let look_at = swivel_trans.translation + dir;
                swivel_trans.look_at(look_at, Vec3::Y);

                //.rotation =
                //    Quat::from_euler(EulerRot::XYZ, before.0, after.1, before.2);
                break;
            }
        }
    }
}

pub fn progress_projectiles(
    mut com: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile), Without<Enemy>>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
    model_assets: Res<ModelAssets>,
    player: Res<PlayerState>,
    pref: Res<Preferences>,
) {
    for (proj_entity, mut proj_trans, mut projectile) in projectiles.iter_mut() {
        proj_trans.translation += projectile.dir * projectile.speed;

        if proj_trans.translation.distance(projectile.dest) < 0.8 {
            projectile.hit = true;
            for (enemy_trans, mut health) in enemies.iter_mut() {
                if enemy_trans.translation.distance(proj_trans.translation)
                    < projectile.blast_radius
                {
                    **health -= projectile.damage * player.wave_upgrade;
                    if **health < 0.0 {
                        let mut ecmds = com.spawn(SceneBundle {
                            scene: model_assets.disc.clone(),
                            transform: Transform::from_translation(
                                enemy_trans.translation + Vec3::Y * 0.5,
                            ),
                            ..Default::default()
                        });
                        ecmds.insert(DiscExplosion {
                            speed: 9.0,
                            size: 4.0,
                            progress: 0.0,
                        });
                        if !pref.less_lights {
                            basic_light(
                                &mut ecmds,
                                Color::rgb(1.0, 0.8, 0.7),
                                300.0,
                                3.0,
                                0.75,
                                vec3(0.0, 0.6, 0.0),
                            );
                        }
                    }
                }
            }
        }
        if projectile.hit {
            projectile.hit_despawn_countdown -= TIMESTEP;
        }
        if projectile.hit_despawn_countdown < 0.0 {
            com.entity(proj_entity).despawn_recursive();
        }
    }
}

pub fn progress_explosions(
    mut com: Commands,
    mut explosions: Query<(Entity, &mut Transform, &mut DiscExplosion)>,
) {
    for (entity, mut trans, mut explosion) in explosions.iter_mut() {
        explosion.progress += TIMESTEP * explosion.speed;
        if explosion.progress >= 1.0 {
            com.entity(entity).despawn_recursive();
        } else {
            trans.scale = Vec3::splat(explosion.progress * explosion.size);
        }
    }
}

pub fn bobble_shockwave_spheres(
    mut shockwave_spheres: Query<(&mut Transform, &mut ShockwaveSphere)>,
    caps: Query<&Cap>,
) {
    for (mut trans, mut sh) in shockwave_spheres.iter_mut() {
        let mut disable_bobble = false;
        for cap in caps.iter() {
            if cap.top_parent == sh.top_parent {
                if cap.progress > 0.1 {
                    disable_bobble = true;
                }
                break;
            }
        }
        if disable_bobble {
            sh.phase = 0.0;
        } else {
            sh.phase += TIMESTEP;
        }
        trans.translation = Vec3::Y * 0.7 + Vec3::Y * 0.2 * (sh.phase * 2.0).sin();
    }
}

pub fn position_caps(mut caps: Query<(&mut Transform, &mut Cap)>) {
    let delta_sec = TIMESTEP;
    for (mut trans, mut cap) in caps.iter_mut() {
        cap.progress = (cap.progress - delta_sec * 2.0).max(0.0);
        trans.translation = cap.direction * cap.progress;
    }
}

#[derive(Component)]
pub struct Projectile {
    pub dir: Vec3,
    pub speed: f32,
    pub dest: Vec3,
    pub enemy: Entity,
    pub damage: f32,
    pub blast_radius: f32,
    pub hit: bool,
    pub hit_despawn_countdown: f32,
}

#[derive(Component)]
pub struct DiscExplosion {
    pub speed: f32,
    pub size: f32,
    pub progress: f32,
}

#[derive(Component, Debug)]
pub struct Swivel {
    top_parent: Entity,
}

#[derive(Component, Debug)]
pub struct ShockwaveSphere {
    top_parent: Entity,
    phase: f32,
}

#[derive(Component, Debug)]
pub struct Cap {
    top_parent: Entity,
    progress: f32,
    direction: Vec3,
}

#[derive(Component, Debug)]
pub struct DiamondLasers {
    top_parent: Entity,
}

#[derive(Component, Debug)]
pub struct LaserBeam {
    top_parent: Entity,
}

#[derive(Component, Debug)]
pub struct Disabled;
