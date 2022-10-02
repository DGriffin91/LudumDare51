use bevy::{math::*, prelude::*};

use crate::{board::GameBoard, player::PlayerState};

#[derive(Component, Deref, DerefMut)]
pub struct EnemyPath(pub Option<(Vec<IVec2>, u32)>);

#[derive(Component, Deref, DerefMut)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Enemy;

pub fn spawn_enemy(
    time: Res<Time>,
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_spawn: Local<f32>,
    b: Res<GameBoard>,
) {
    let since_startup = time.seconds_since_startup() as f32;
    if since_startup - *last_spawn > 1.0 {
        *last_spawn = since_startup;
        com.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.5,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
            transform: Transform::from_translation(b.ls_to_ws_vec3(b.start) + vec3(0.0, 0.5, 0.0)),
            ..default()
        })
        .insert(EnemyPath(None))
        .insert(Health(1.0))
        .insert(Enemy);
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
