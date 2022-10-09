use bevy::{math::*, prelude::*};

use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use rand::Rng;

use crate::{
    assets::ModelAssets,
    audio::{AudioEvents, EXPLOSION_SOUND},
    basic_light,
    board::GameBoard,
    player::{PlayerState, GAMESETTINGS},
    schedule::TIMESTEP,
    turrets::DiscExplosion,
    ui::Preferences,
    GameRng,
};

pub struct EnemiesPlugin;
impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(LastSpawns::default());
    }
}

#[derive(Component, Default)]
pub struct EnemyPath {
    pub path: Option<(Vec<IVec2>, u32)>,
    pub new_rand_loc_timer: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Enemy {
    speed: f32,
}

#[derive(Component)]
pub struct FlyingEnemy {
    dest: Vec3,
    new_rand_loc_timer: f32,
}

#[derive(Default)]
pub struct LastSpawns {
    rolling_enemy: f32,
    rolling_enemy2: f32,
    flying_enemy: f32,
}

pub(crate) fn spawn_rolling_enemy(
    mut com: Commands,
    mut last_spawns: ResMut<LastSpawns>,
    b: Res<GameBoard>,
    model_assets: Res<ModelAssets>,
    player: Res<PlayerState>,
    pref: Res<Preferences>,
) {
    if !player.alive() {
        return;
    }

    if player.level < 20.0 {
        return;
    }
    if b.has_enemy[0] {
        return;
    }
    let since_startup = TIMESTEP * player.step as f32;
    if since_startup - last_spawns.rolling_enemy
        > (GAMESETTINGS.rolling_enemy_spawn_speed - player.spawn_rate_cut())
            .max(GAMESETTINGS.rolling_enemy_max_spawn_speed)
    {
        last_spawns.rolling_enemy = since_startup;
        let mut ecmds = com.spawn();

        ecmds
            .insert(EnemyPath::default())
            .insert(Health(
                GAMESETTINGS.rolling_enemy_health * player.enemy_health_mult(),
            ))
            .insert(Enemy {
                speed: GAMESETTINGS.rolling_enemy_speed + player.enemy_speed_boost(),
            });

        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.1, 0.1),
            30.0,
            1.5 * pref.light_r,
            0.5,
            vec3(0.0, 0.4, -0.5),
        );

        ecmds.insert_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.rolling_enemy.clone(),
                transform: Transform::from_translation(b.ls_to_ws_vec3(b.start)),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        });
    }
}

pub(crate) fn spawn_rolling_enemy2(
    mut com: Commands,
    mut last_spawns: ResMut<LastSpawns>,
    b: Res<GameBoard>,
    model_assets: Res<ModelAssets>,
    player: Res<PlayerState>,
    pref: Res<Preferences>,
) {
    if !player.alive() {
        return;
    }
    if player.level < 1.0 {
        return;
    }
    if b.has_enemy[0] {
        return;
    }
    let since_startup = TIMESTEP * player.step as f32;
    if since_startup - last_spawns.rolling_enemy2
        > (GAMESETTINGS.rolling_enemy_2_spawn_speed - player.spawn_rate_cut())
            .max(GAMESETTINGS.rolling_enemy_2_max_spawn_speed)
    {
        last_spawns.rolling_enemy2 = since_startup;
        let mut ecmds = com.spawn();

        ecmds
            .insert(EnemyPath::default())
            .insert(Health(
                GAMESETTINGS.rolling_enemy_2_health * player.enemy_health_mult(),
            ))
            .insert(Enemy {
                speed: GAMESETTINGS.rolling_enemy_2_speed + player.enemy_speed_boost(),
            });

        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.1, 0.3),
            30.0,
            1.5 * pref.light_r,
            0.5,
            vec3(0.0, 0.4, -0.5),
        );

        ecmds.insert_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.rolling_enemy_2.clone(),
                transform: Transform::from_translation(b.ls_to_ws_vec3(b.start)),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        });
    }
}

pub(crate) fn spawn_flying_enemy(
    mut com: Commands,
    mut last_spawns: ResMut<LastSpawns>,
    b: Res<GameBoard>,
    model_assets: Res<ModelAssets>,
    player: Res<PlayerState>,
    pref: Res<Preferences>,
    mut rng: ResMut<GameRng>,
) {
    if !player.alive() {
        return;
    }
    if player.level < 10.0 {
        return;
    }
    if b.has_enemy[0] {
        return;
    }
    let since_startup = TIMESTEP * player.step as f32;
    if since_startup - last_spawns.flying_enemy
        > (GAMESETTINGS.flying_enemy_spawn_speed
            - player.spawn_rate_cut() * ((player.level - 20.0) * 0.25).clamp(1.0, 50.0))
        .max(GAMESETTINGS.flying_enemy_max_spawn_speed)
    {
        last_spawns.flying_enemy = since_startup;
        let mut ecmds = com.spawn();

        ecmds
            .insert(Health(
                GAMESETTINGS.flying_enemy_health * player.enemy_health_mult(),
            ))
            .insert(Enemy {
                speed: GAMESETTINGS.flying_enemy_speed + player.enemy_speed_boost(),
            })
            .insert(FlyingEnemy {
                dest: b.ls_to_ws_vec3(b.dest),
                new_rand_loc_timer: 0.0,
            });

        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.1, 0.1),
            200.0,
            2.5 * pref.light_r,
            0.2,
            vec3(0.0, 0.3, -0.2),
        );

        // Random pos off screen
        let rnd_offset = vec3(
            rng.gen_range(-15.0..-5.0) as f32,
            0.0,
            rng.gen_range(-15.0..-5.0) as f32,
        );

        ecmds.insert_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.flying_enemy.clone(),
                transform: Transform::from_translation(
                    b.ls_to_ws_vec3(b.start) + Vec3::Y * 2.0 + rnd_offset,
                ),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        });
    }
}

pub(crate) fn destroy_enemies(
    mut com: Commands,
    enemies: Query<(Entity, &Health), With<Enemy>>,
    mut player: ResMut<PlayerState>,
    mut audio_events: ResMut<AudioEvents>,
) {
    if !player.alive() {
        return;
    }
    for (entity, health) in enemies.iter() {
        if health.0 < 0.0 {
            com.entity(entity).despawn_recursive();
            player.credits += GAMESETTINGS.credits_for_kill;
            player.kills += 1;
            **audio_events |= EXPLOSION_SOUND;
        }
    }
}

pub(crate) fn update_board_has_enemy(
    enemies: Query<&Transform, With<Enemy>>,
    mut b: ResMut<GameBoard>,
) {
    b.reset_has_enemy();
    for trans in enemies.iter() {
        let idx = b.ls_to_idx(b.ws_vec3_to_ls(trans.translation));
        b.has_enemy[idx] = true;
    }
}

#[derive(Component)]
pub struct PathInd;

pub(crate) fn update_enemy_paths(
    b: Res<GameBoard>,
    mut enemies: Query<(&Transform, &mut EnemyPath)>,
    player: Res<PlayerState>,
) {
    if !player.alive() {
        return;
    }
    for (trans, mut enemy_path) in enemies.iter_mut() {
        enemy_path.path = b.path(b.ws_vec3_to_ls(trans.translation), b.dest);
    }
}

pub(crate) fn update_enemy_postgame_paths(
    b: Res<GameBoard>,
    mut enemies: Query<(&Transform, &mut EnemyPath)>,
    player: Res<PlayerState>,
    mut rng: ResMut<GameRng>,
) {
    if player.alive() {
        return;
    }

    for (trans, mut enemy_path) in enemies.iter_mut() {
        enemy_path.new_rand_loc_timer -= TIMESTEP;
        if enemy_path.new_rand_loc_timer < 0.0 {
            enemy_path.new_rand_loc_timer += rng.gen_range(4.0..6.0);
            let rnd = vec3(
                rng.gen_range(-12.0..12.0) as f32,
                0.0,
                rng.gen_range(-12.0..12.0) as f32,
            );
            enemy_path.path = b.path(b.ws_vec3_to_ls(trans.translation), b.ws_vec3_to_ls(rnd));
        } else if let Some(path) = &enemy_path.path {
            if let Some(last) = path.0.last() {
                enemy_path.path = b.path(b.ws_vec3_to_ls(trans.translation), *last);
            }
        }
    }
}

#[allow(dead_code)]
pub(crate) fn debug_show_enemy_path(
    b: Res<GameBoard>,
    enemies: Query<(&Transform, &EnemyPath)>,
    mut com: Commands,
    path_ind: Query<Entity, With<PathInd>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &path_ind {
        com.entity(entity).despawn_recursive();
    }
    if let Some((_, enemy_path)) = enemies.iter().next() {
        for path in &enemy_path.path {
            for p in &path.0 {
                com.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 0.5,
                        ..default()
                    })),
                    material: materials.add(Color::rgb(0.0, 0.5, 0.0).into()),
                    transform: Transform::from_translation(
                        b.ls_to_ws_vec3(*p) + vec3(0.0, 0.5, 0.0),
                    ),
                    ..default()
                })
                .insert(PathInd);
            }
        }
    }
}

pub(crate) fn move_enemy_along_path(
    b: Res<GameBoard>,
    mut enemies: Query<(&mut Transform, &mut EnemyPath, &Enemy)>,
) {
    for (mut enemy_trans, enemy_path, enemy) in enemies.iter_mut() {
        if let Some(path) = &enemy_path.path {
            if path.0.len() > 1 {
                let p = enemy_trans.translation;
                let a = b.ls_to_ws_vec3(path.0[1]);
                let next_pos = a;
                if !b.has_enemy[b.ls_to_idx(b.ws_vec3_to_ls(next_pos))] {
                    enemy_trans.translation += (next_pos - p).normalize() * TIMESTEP * enemy.speed;
                }
                enemy_trans.look_at(next_pos, Vec3::Y);
            }
        }
    }
}

pub(crate) fn check_enemy_at_dest(
    mut com: Commands,
    b: Res<GameBoard>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut player: ResMut<PlayerState>,
    model_assets: Res<ModelAssets>,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (enemy_entity, enemy_trans) in enemies.iter() {
        if enemy_trans.translation.distance(b.ls_to_ws_vec3(b.dest)) < 1.0 {
            player.health -= 0.1;
            com.entity(enemy_entity).despawn_recursive();
            let mut ecmds = com.spawn_bundle(SceneBundle {
                scene: model_assets.disc.clone(),
                transform: Transform::from_translation(enemy_trans.translation + Vec3::Y * 0.5),
                ..Default::default()
            });
            ecmds.insert(DiscExplosion {
                speed: 9.0,
                size: 4.0,
                progress: 0.0,
            });
            **audio_events |= EXPLOSION_SOUND;
            basic_light(
                &mut ecmds,
                Color::rgb(1.0, 0.8, 0.7),
                300.0,
                3.5,
                0.75,
                vec3(0.0, 1.6, 0.0),
            );
        }
    }
}

pub(crate) fn move_flying_enemy(mut enemies: Query<(&mut Transform, &mut FlyingEnemy, &Enemy)>) {
    for (mut enemy_trans, fly_enemy, enemy) in enemies.iter_mut() {
        enemy_trans.look_at(fly_enemy.dest, Vec3::Y);
        let dir = (fly_enemy.dest - enemy_trans.translation).normalize();
        enemy_trans.translation += dir * enemy.speed * TIMESTEP;
    }
}

pub(crate) fn check_flying_enemy_at_dest(
    mut com: Commands,
    b: Res<GameBoard>,
    enemies: Query<(Entity, &Transform), With<FlyingEnemy>>,
    mut player: ResMut<PlayerState>,
    model_assets: Res<ModelAssets>,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (enemy_entity, enemy_trans) in enemies.iter() {
        if enemy_trans.translation.distance(b.ls_to_ws_vec3(b.dest)) < 0.5 {
            player.health -= 0.05;
            com.entity(enemy_entity).despawn_recursive();
            let mut ecmds = com.spawn_bundle(SceneBundle {
                scene: model_assets.disc.clone(),
                transform: Transform::from_translation(enemy_trans.translation + Vec3::Y * 0.5),
                ..Default::default()
            });
            ecmds.insert(DiscExplosion {
                speed: 9.0,
                size: 4.0,
                progress: 0.0,
            });
            **audio_events |= EXPLOSION_SOUND;
            basic_light(
                &mut ecmds,
                Color::rgb(1.0, 0.8, 0.7),
                300.0,
                3.5,
                0.75,
                vec3(0.0, 1.6, 0.0),
            );
        }
    }
}

pub(crate) fn update_flying_enemy_postgame_dest(
    mut enemies: Query<(Entity, &mut Transform, &mut FlyingEnemy, &Enemy)>,
    player: Res<PlayerState>,
    mut rng: ResMut<GameRng>,
) {
    if player.alive() {
        return;
    }

    for (_enemy_entity, mut _enemy_trans, mut fly_enemy, _enemy) in enemies.iter_mut() {
        fly_enemy.new_rand_loc_timer -= TIMESTEP;
        if fly_enemy.new_rand_loc_timer < 0.0 {
            fly_enemy.new_rand_loc_timer += rng.gen_range(1.0..5.0);
            let rnd = vec3(
                rng.gen_range(-35.0..35.0) as f32,
                2.0,
                rng.gen_range(-35.0..35.0) as f32,
            );
            fly_enemy.dest = rnd;
        }
    }
}
