use bevy::{math::*, prelude::*};
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

use crate::{assets::ModelAssets, board::GameBoard, player::PlayerState, turrets::basic_light};

#[derive(Component, Deref, DerefMut)]
pub struct EnemyPath(pub Option<(Vec<IVec2>, u32)>);

#[derive(Component, Deref, DerefMut)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Enemy {
    speed: f32,
}

#[derive(Component)]
pub struct FlyingEnemy {
    dest: Vec3,
}

pub fn spawn_enemy(
    time: Res<Time>,
    mut com: Commands,
    mut last_spawn: Local<f32>,
    b: Res<GameBoard>,
    model_assets: Res<ModelAssets>,
) {
    let since_startup = time.seconds_since_startup() as f32;
    if since_startup - *last_spawn > 1.0 {
        *last_spawn = since_startup;
        let mut ecmds = com.spawn();

        ecmds
            .insert(EnemyPath(None))
            .insert(Health(1.0))
            .insert(Enemy { speed: 2.0 });

        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.1, 0.1),
            30.0,
            2.0,
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

pub fn spawn_flying_enemy(
    time: Res<Time>,
    mut com: Commands,
    mut last_spawn: Local<f32>,
    b: Res<GameBoard>,
    model_assets: Res<ModelAssets>,
) {
    let since_startup = time.seconds_since_startup() as f32;
    if since_startup - *last_spawn > 1.0 {
        *last_spawn = since_startup;
        let mut ecmds = com.spawn();

        ecmds
            .insert(Health(0.5))
            .insert(Enemy { speed: 4.0 })
            .insert(FlyingEnemy {
                dest: b.ls_to_ws_vec3(b.dest),
            });

        basic_light(
            &mut ecmds,
            Color::rgb(1.0, 0.1, 0.1),
            200.0,
            3.5,
            0.2,
            vec3(0.0, 0.3, -0.2),
        );

        ecmds.insert_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.flying_enemy.clone(),
                transform: Transform::from_translation(b.ls_to_ws_vec3(b.start) + Vec3::Y * 2.0),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        });
    }
}

pub fn destroy_enemies(
    mut com: Commands,
    enemies: Query<(Entity, &Health), With<Enemy>>,
    mut player: ResMut<PlayerState>,
) {
    for (entity, health) in enemies.iter() {
        if health.0 < 0.0 {
            com.entity(entity).despawn_recursive();
            player.credits += 100; //100 credits per kill
            player.kills += 1;
        }
    }
}

pub fn update_board_has_enemy(enemies: Query<&Transform, With<Enemy>>, mut b: ResMut<GameBoard>) {
    b.reset_has_enemy();
    for trans in enemies.iter() {
        let idx = b.ls_to_idx(b.ws_vec3_to_ls(trans.translation));
        b.has_enemy[idx] = true;
    }
}

#[derive(Component)]
pub struct PathInd;

pub fn update_enemy_paths(
    b: Res<GameBoard>,
    mut enemies: Query<(&Transform, &mut EnemyPath)>,
    //mut com: Commands,
    //path_ind: Query<Entity, With<PathInd>>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (trans, mut enemy_path) in enemies.iter_mut() {
        enemy_path.0 = b.path(b.ws_vec3_to_ls(trans.translation));
    }
    // DEBUG PATH SPHERES
    //for entity in &path_ind {
    //    com.entity(entity).despawn_recursive();
    //}
    //if let Some((_, enemy_path)) = enemies.iter().next() {
    //    for path in &enemy_path.0 {
    //        for p in &path.0 {
    //            com.spawn_bundle(PbrBundle {
    //                mesh: meshes.add(Mesh::from(shape::UVSphere {
    //                    radius: 0.5,
    //                    ..default()
    //                })),
    //                material: materials.add(Color::rgb(0.0, 0.5, 0.0).into()),
    //                transform: Transform::from_translation(
    //                    b.ls_to_ws_vec3(*p) + vec3(0.0, 0.5, 0.0),
    //                ),
    //                ..default()
    //            })
    //            .insert(PathInd);
    //        }
    //    }
    //}
}

pub fn move_enemy_along_path(
    time: Res<Time>,
    b: Res<GameBoard>,
    mut enemies: Query<(&mut Transform, &mut EnemyPath, &Enemy)>,
) {
    for (mut trans, enemy_path, enemy) in enemies.iter_mut() {
        if let Some(path) = &enemy_path.0 {
            let p = trans.translation;
            let a = b.ls_to_ws_vec3(path.0[1]);
            let next_pos = a;
            if !b.has_enemy[b.ls_to_idx(b.ws_vec3_to_ls(next_pos))] {
                trans.translation +=
                    (next_pos - p).normalize() * time.delta_seconds() * enemy.speed;
            }
            trans.look_at(next_pos, Vec3::Y);
        }
    }
}

pub fn move_flying_enemy(
    time: Res<Time>,
    mut enemies: Query<(&mut Transform, &mut FlyingEnemy, &Enemy)>,
) {
    for (mut enemy_trans, fly_enemy, enemy) in enemies.iter_mut() {
        enemy_trans.look_at(fly_enemy.dest, Vec3::Y);
        let dir = (fly_enemy.dest - enemy_trans.translation).normalize();
        enemy_trans.translation += dir * enemy.speed * time.delta_seconds();
    }
}
