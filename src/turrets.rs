use std::f32::INFINITY;
use std::time::Duration;

use bevy::ecs::system::EntityCommands;
use bevy::math::*;
use bevy::prelude::*;
use bevy_scene_hook::HookedSceneBundle;
use bevy_scene_hook::SceneHook;

use crate::{
    assets::ModelAssets,
    enemies::{Enemy, Health},
};

#[derive(Clone, Copy, Component, PartialEq, Eq)]
pub enum Turret {
    Laser,
    Shockwave,
}

impl Turret {
    pub fn cost(&self) -> u64 {
        match self {
            Turret::Laser => 100,
            Turret::Shockwave => 200,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AttackDamage(pub f32);

#[derive(Component, Deref, DerefMut)]
pub struct Cooldown(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct Range(f32);

impl Turret {
    pub fn spawn_shockwave_turret(
        com: &mut Commands,
        trans: Vec3,
        model_assets: &ModelAssets,
    ) -> (Turret, bevy::prelude::Entity) {
        let mut ecmds = com.spawn();
        let entity_id = ecmds.id();

        ecmds
            .insert(AttackDamage(0.05))
            .insert(Cooldown(Timer::new(Duration::from_secs_f32(0.5), true)))
            .insert(Range(4.0))
            .insert(Turret::Shockwave);
        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.2, 1.0),
            200.0,
            3.0,
            0.75,
            vec3(0.0, 0.6, 0.0),
        );

        ecmds.insert_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.shockwave_turret.clone(),
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |entity, cmds| {
                if let Some(name) = entity.get::<Name>() {
                    if name.contains("BobbleSphere") {
                        cmds.insert(ShockwaveSphere {
                            top_parent: entity_id.clone(),
                            phase: 0.0,
                        });
                    }
                    if name.contains("Top Cap") {
                        cmds.insert(Cap {
                            top_parent: entity_id.clone(),
                            progress: 0.0,
                            direction: Vec3::Y * 0.1,
                        });
                    }
                    if name.contains("Bottom Cap") {
                        cmds.insert(Cap {
                            top_parent: entity_id.clone(),
                            progress: 0.0,
                            direction: -Vec3::Y * 0.1,
                        });
                    }
                }
            }),
        });

        (Turret::Shockwave, entity_id)
    }

    pub fn spawn_laser_turret(
        com: &mut Commands,
        trans: Vec3,
        model_assets: &ModelAssets,
    ) -> (Turret, bevy::prelude::Entity) {
        let mut ecmds = com.spawn();
        let entity_id = ecmds.id();
        ecmds
            .insert(AttackDamage(0.02))
            .insert(Cooldown(Timer::new(Duration::from_secs_f32(0.5), true)))
            .insert(Range(4.0))
            .insert(Turret::Laser);
        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.2, 0.1),
            200.0,
            3.0,
            0.75,
            vec3(0.0, 1.0, 0.0),
        );

        ecmds.insert_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.laser_turret.clone(),
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |entity, cmds| {
                if let Some(name) = entity.get::<Name>() {
                    if name.contains("Head") {
                        cmds.insert(Swivel {
                            top_parent: entity_id.clone(),
                        });
                    }
                }
            }),
        });

        (Turret::Laser, entity_id)
    }
}

pub fn turret_fire(
    mut com: Commands,
    time: Res<Time>,
    mut turrets: Query<(
        Entity,
        &Transform,
        &AttackDamage,
        &Range,
        &mut Cooldown,
        &Turret,
    )>,
    mut enemies: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
    model_assets: Res<ModelAssets>,
    mut caps: Query<&mut Cap>,
) {
    for (turret_entity, turret_trans, damage, range, mut cooldown, turret) in turrets.iter_mut() {
        cooldown.tick(time.delta());

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
                Turret::Laser => {
                    if let Some(entity) = closest {
                        if let Ok((entity, enemy_trans, mut _health)) = enemies.get_mut(entity) {
                            cooldown.reset();
                            let turret_head_trans = turret_trans.translation + Vec3::Y * 1.7;
                            let fire_dir =
                                (enemy_trans.translation - turret_head_trans).normalize();
                            let mut ecmds = com.spawn();

                            ecmds.insert_bundle(SceneBundle {
                                scene: model_assets.projectile_laser_blast.clone(),
                                transform: Transform::from_translation(turret_head_trans)
                                    .looking_at(enemy_trans.translation, Vec3::Y),
                                ..default()
                            });
                            ecmds.insert(Projectile {
                                dir: fire_dir,
                                speed: 0.5,
                                dest: enemy_trans.translation,
                                enemy: entity.clone(),
                                damage: damage.0,
                                blast_radius: 1.5,
                                hit: false,
                                hit_despawn_countdown: 1.0,
                            });
                            basic_light(
                                &mut ecmds,
                                Color::rgb(1.0, 0.0, 0.0),
                                700.0,
                                2.0,
                                1.0,
                                Vec3::Y * -0.5,
                            );
                        }
                    }
                }
                Turret::Shockwave => {
                    for (_entity, enemy_trans, mut health) in enemies.iter_mut() {
                        let dist = enemy_trans.translation.distance(turret_trans.translation);
                        if dist < **range {
                            for mut cap in caps.iter_mut() {
                                if cap.top_parent == turret_entity {
                                    cap.progress = 1.0;
                                }
                            }
                            cooldown.reset();
                            health.0 -= damage.0 * (1.0 / dist.max(1.0));
                            let mut ecmds = com.spawn_bundle(SceneBundle {
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
                            basic_light(
                                &mut ecmds,
                                Color::rgb(1.0, 0.0, 1.0),
                                500.0,
                                4.5,
                                1.0,
                                Vec3::Y,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn basic_light(
    cmds: &mut EntityCommands,
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
    trans: Vec3,
) {
    cmds.add_children(|parent| {
        parent.spawn_bundle(PointLightBundle {
            point_light: PointLight {
                color,
                intensity,
                range,
                radius,
                ..default()
            },
            transform: Transform::from_translation(trans),
            ..default()
        });
    });
}

pub fn laser_point_at_enemy(
    mut turrets: Query<(Entity, &mut Transform, &Range), (With<Turret>, Without<Swivel>)>,
    mut swivels: Query<(&mut Transform, &Swivel), Without<Turret>>,
    mut enemies: Query<&Transform, (With<Enemy>, (Without<Turret>, Without<Swivel>))>,
) {
    for (turret_entity, mut turret_trans, _range) in turrets.iter_mut() {
        let mut closest = Vec3::ZERO;
        let mut closest_dist = INFINITY;
        for enemy_trans in enemies.iter_mut() {
            let dist = turret_trans.translation.distance(enemy_trans.translation);
            if dist < closest_dist {
                closest = enemy_trans.translation;
                closest_dist = dist;
            }
        }
        let dir = (closest - turret_trans.translation - Vec3::Y * 1.7).normalize();
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
    time: Res<Time>,
    mut com: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile), Without<Enemy>>,
    mut enemies: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
    model_assets: Res<ModelAssets>,
) {
    for (proj_entity, mut proj_trans, mut projectile) in projectiles.iter_mut() {
        proj_trans.translation += projectile.dir * projectile.speed;

        if proj_trans.translation.distance(projectile.dest) < 0.5 {
            projectile.hit = true;
            for (entity, enemy_trans, mut health) in enemies.iter_mut() {
                if enemy_trans.translation.distance(proj_trans.translation)
                    < projectile.blast_radius
                {
                    **health -= projectile.damage;
                    if **health < 0.0 {
                        com.entity(entity).despawn_recursive();
                        com.spawn_bundle(SceneBundle {
                            scene: model_assets.disc.clone(),
                            transform: proj_trans.clone(),
                            ..Default::default()
                        })
                        .insert(DiscExplosion {
                            speed: 9.0,
                            size: 4.0,
                            progress: 0.0,
                        });
                    }
                }
            }
        }
        if projectile.hit {
            projectile.hit_despawn_countdown -= time.delta_seconds();
        }
        if projectile.hit_despawn_countdown < 0.0 {
            com.entity(proj_entity).despawn_recursive();
        }
    }
}

pub fn progress_explosions(
    time: Res<Time>,
    mut com: Commands,
    mut explosions: Query<(Entity, &mut Transform, &mut DiscExplosion)>,
) {
    for (entity, mut trans, mut explosion) in explosions.iter_mut() {
        explosion.progress += time.delta_seconds() * explosion.speed;
        if explosion.progress >= 1.0 {
            com.entity(entity).despawn_recursive();
        } else {
            trans.scale = Vec3::splat(explosion.progress * explosion.size);
        }
    }
}

pub fn bobble_shockwave_spheres(
    time: Res<Time>,
    mut shockwave_spheres: Query<(&mut Transform, &mut ShockwaveSphere)>,
) {
    for (mut trans, mut sh) in shockwave_spheres.iter_mut() {
        sh.phase += time.delta_seconds();
        trans.translation = Vec3::Y * 0.7 + Vec3::Y * 0.2 * (sh.phase * 2.0).sin() as f32;
    }
}

pub fn position_caps(time: Res<Time>, mut caps: Query<(&mut Transform, &mut Cap)>) {
    let delta_sec = time.delta_seconds();
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