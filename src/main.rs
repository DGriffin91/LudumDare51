#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::TAU;

use assets::{FontAssets, GameState, ModelAssets};
use bevy::{
    asset::AssetServerSettings,
    math::*,
    prelude::*,
    render::camera::Projection,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};
use bevy_mod_picking::*;
use bevy_mod_raycast::RayCastMesh;

use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};
use board::GameBoard;
use enemies::{
    destroy_enemies, move_enemy_along_path, move_flying_enemy, spawn_flying_enemy,
    spawn_rolling_enemy, spawn_rolling_enemy2, update_board_has_enemy, update_enemy_paths,
};
use iyes_loopless::prelude::*;
use player::{MyRaycastSet, PlayerPlugin};
use turrets::{
    basic_light, bobble_shockwave_spheres, laser_point_at_enemy, position_caps,
    progress_explosions, progress_projectiles, turret_fire,
};
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
    //.add_plugin(EditorPlugin);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    app.add_enter_system(GameState::RunLevel, setup)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::RunLevel)
                .label("pre")
                .with_system(spawn_rolling_enemy)
                .with_system(spawn_rolling_enemy2)
                .with_system(update_enemy_paths)
                .with_system(move_enemy_along_path)
                .with_system(turret_fire)
                .with_system(destroy_enemies)
                .with_system(update_board_has_enemy)
                .with_system(laser_point_at_enemy)
                .with_system(progress_projectiles)
                .with_system(progress_explosions)
                .with_system(bobble_shockwave_spheres)
                .with_system(position_caps)
                .with_system(move_flying_enemy)
                .with_system(spawn_flying_enemy)
                .into(),
        );

    app.run();
}

#[derive(Component)]
pub struct Board;

/// set up a simple 3D scene
fn setup(
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    model_assets: Res<ModelAssets>,
) {
    let b = GameBoard::new(ivec2(-12, -12), [24, 24], ivec2(0, 0), ivec2(22, 22));
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
            illuminance: 20000.0,
            //shadows_enabled: true,
            color: Color::rgb(1.0, 1.0, 1.0),
            ..default()
        },
        transform: Transform::from_translation(vec3(0.0, 5.0, 0.0)).with_rotation(
            Quat::from_euler(EulerRot::XYZ, TAU * 0.5, -TAU * 0.25, TAU * 0.25),
        ),
        ..default()
    });
    let side = 3.0;
    // camera
    com.spawn_bundle(Camera3dBundle {
        transform: (Transform::from_translation(vec3(48.0 + side, 48.0, 48.0 - side)))
            .looking_at(vec3(side, -2.0, -side), Vec3::Y),
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 16f32.to_radians(),
            ..default()
        }),
        ..default()
    })
    .insert_bundle(PickingCameraBundle::default())
    .insert(RayCastSource::<MyRaycastSet>::new());

    // Main Base

    let mut ecmds = com.spawn();
    ecmds.insert(MainBase);
    basic_light(
        &mut ecmds,
        Color::rgb(1.0, 0.1, 1.0),
        400.0,
        6.0,
        2.0,
        vec3(0.0, 2.0, 0.0),
    );

    ecmds.insert_bundle(HookedSceneBundle {
        scene: SceneBundle {
            scene: model_assets.base.clone(),
            transform: Transform::from_translation(b.ls_to_ws_vec3(b.dest)),
            ..default()
        },
        hook: SceneHook::new(move |_entity, _cmds| {}),
    });

    // Insert Board
    com.insert_resource(b);
}

#[derive(Component)]
struct MainBase;
