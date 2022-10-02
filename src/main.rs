#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::TAU;

use assets::{FontAssets, GameState, ModelAssets};
use bevy::{
    asset::AssetServerSettings,
    math::*,
    prelude::*,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};
use bevy_mod_picking::*;
use bevy_mod_raycast::RayCastMesh;

use bevy_scene_hook::HookPlugin;
use board::GameBoard;
use enemies::{destroy_enemies, spawn_enemy, update_board_has_enemy, EnemyPath};
use iyes_loopless::prelude::*;
use player::{MyRaycastSet, PlayerPlugin};
use turrets::{laser_point_at_enemy, progress_explosions, progress_projectiles, turret_fire};
use ui::GameUI;
pub mod assets;
pub mod board;
pub mod enemies;
pub mod player;
pub mod turrets;
pub mod ui;

fn main() {
    let mut app = App::new();
    app.add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::RunLevel)
                .with_collection::<FontAssets>()
                .with_collection::<ModelAssets>(),
        );

    app.insert_resource(WindowDescriptor {
        title: "LD51".to_string(),
        width: 1280.0,
        height: 720.0,
        position: WindowPosition::Automatic,
        resize_constraints: WindowResizeConstraints {
            min_width: 256.0,
            min_height: 256.0,
            ..Default::default()
        },
        scale_factor_override: Some(1.0), //Needed for some mobile devices, but disables scaling
        present_mode: PresentMode::AutoVsync,
        resizable: true,
        decorations: true,
        cursor_locked: false,
        cursor_visible: true,
        mode: WindowMode::Windowed,
        transparent: false,
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
    })
    .insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(DefaultPlugins)
    .add_plugin(GameUI)
    .add_plugin(PlayerPlugin)
    .add_plugin(HookPlugin);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    app.add_enter_system(GameState::RunLevel, setup)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::RunLevel)
                .label("pre")
                .with_system(spawn_enemy)
                .with_system(update_enemy_paths)
                .with_system(move_enemy_along_path)
                .with_system(turret_fire)
                .with_system(destroy_enemies)
                .with_system(update_board_has_enemy)
                .with_system(laser_point_at_enemy)
                .with_system(progress_projectiles)
                .with_system(progress_explosions)
                .into(),
        );

    app.run();
}

#[derive(Component)]
struct PathInd;

fn update_enemy_paths(
    mut com: Commands,
    b: Res<GameBoard>,
    mut enemies: Query<(&Transform, &mut EnemyPath)>,
    path_ind: Query<Entity, With<PathInd>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (trans, mut enemy_path) in enemies.iter_mut() {
        enemy_path.0 = b.path(b.ws_vec3_to_ls(trans.translation));
    }
    for entity in &path_ind {
        com.entity(entity).despawn_recursive();
    }
    if let Some((_, enemy_path)) = enemies.iter().next() {
        for path in &enemy_path.0 {
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

fn move_enemy_along_path(
    time: Res<Time>,
    b: Res<GameBoard>,
    mut enemies: Query<(&mut Transform, &mut EnemyPath)>,
) {
    for (mut trans, enemy_path) in enemies.iter_mut() {
        if let Some(path) = &enemy_path.0 {
            let p = trans.translation;
            let a = b.ls_to_ws_vec3(path.0[1]) + vec3(0.0, 0.5, 0.0);
            let next_pos = a;
            if !b.has_enemy[b.ls_to_idx(b.ws_vec3_to_ls(next_pos))] {
                trans.translation += (next_pos - p).normalize() * time.delta_seconds() * 2.0;
            }
        }
    }
}

#[derive(Component)]
pub struct Board;

/// set up a simple 3D scene
fn setup(
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    com.insert_resource(GameBoard::new(
        ivec2(-12, -12),
        [24, 24],
        ivec2(0, 0),
        ivec2(22, 22),
    ));
    //com.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
    // plane
    com.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 24.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.2, 0.2),
            perceptual_roughness: 0.4,
            ..default()
        }),
        ..default()
    })
    .insert_bundle(PickableBundle::default())
    .insert(Board)
    .insert(RayCastMesh::<MyRaycastSet>::default());

    // light
    com.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 30000.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 0.95, 0.5),
            ..default()
        },
        transform: Transform::from_translation(vec3(0.0, 5.0, 0.0)).with_rotation(
            Quat::from_euler(EulerRot::XYZ, TAU * 0.5, TAU * 0.125, TAU * 0.25),
        ),
        ..default()
    });

    // camera
    com.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 24.0, 18.0).looking_at(vec3(0.0, 0.0, 3.0), Vec3::Y),
        ..default()
    })
    .insert_bundle(PickingCameraBundle::default())
    .insert(RayCastSource::<MyRaycastSet>::new());
}
