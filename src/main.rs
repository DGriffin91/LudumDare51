#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::TAU;

use assets::{AudioAssets, FontAssets, ModelAssets};
use audio::GameAudioPlugin;
use bevy::{
    ecs::{schedule::ShouldRun, system::EntityCommands},
    math::*,
    prelude::*,
    render::camera::Projection,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};

use bevy_mod_raycast::{RaycastMesh, RaycastSource};

use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};
use board::GameBoard;

use enemies::{EnemiesPlugin, Enemy, LastSpawns};
use iyes_loopless::prelude::*;
use player::{MyRaycastSet, PlayerState};

use rand_pcg::Pcg32;
use turrets::{Disabled, Projectile, Turret};
use ui::GameUI;
pub mod action;
pub mod assets;
pub mod audio;
pub mod board;
pub mod enemies;
pub mod player;
pub mod schedule;
pub mod turrets;
pub mod ui;

fn main() {
    let mut app = App::new();
    app.add_loopless_state(GameState::AssetLoading)
        .add_loopless_state(PausedState::Unpaused)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::RunLevel)
                .with_collection::<FontAssets>()
                .with_collection::<ModelAssets>()
                .with_collection::<AudioAssets>(),
        );

    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "LD51".to_string(),
                        width: 1280.0,
                        height: 720.0,
                        position: WindowPosition::Automatic,
                        resize_constraints: WindowResizeConstraints {
                            min_width: 960.0,
                            min_height: 480.0,
                            ..Default::default()
                        },
                        scale_factor_override: Some(1.0), //Needed for some mobile devices, but disables scaling
                        present_mode: PresentMode::AutoVsync,
                        resizable: true,
                        decorations: true,
                        cursor_visible: true,
                        mode: WindowMode::Windowed,
                        transparent: false,
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        ..default()
                    },
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                }),
        )
        .insert_resource(GameBoard::default())
        .insert_resource(RestartGame::default())
        .insert_resource(GameRng::default())
        .add_plugin(HookPlugin);

    app.add_plugin(GameUI)
        .add_plugin(EnemiesPlugin)
        .add_plugin(GameAudioPlugin);
    schedule::setup_schedule(&mut app);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    app.add_enter_system(GameState::RunLevel, setup_level)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::RunLevel)
                .with_system(destroy_base_disable_turrets)
                .into(),
        );

    app.run();
}

#[derive(Resource, Deref, DerefMut)]
pub struct GameRng(pub Pcg32);

impl Default for GameRng {
    fn default() -> Self {
        GameRng(Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7))
    }
}

#[derive(Component)]
pub struct Board;

/// set up a simple 3D scene
fn setup_level(
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    model_assets: Res<ModelAssets>,
    b: Res<GameBoard>,
) {
    // com.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
    // plane
    com.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 24.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.2, 0.2),
            perceptual_roughness: 0.4,
            ..default()
        }),
        ..default()
    })
    .insert(Board)
    .insert(RaycastMesh::<MyRaycastSet>::default());

    com.spawn(SceneBundle {
        scene: model_assets.board.clone(),
        transform: Transform::from_translation(vec3(0.0, -0.1, 0.0)),
        ..default()
    });

    // light
    com.spawn(DirectionalLightBundle {
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
    com.spawn(Camera3dBundle {
        transform: (Transform::from_translation(vec3(48.0 + side, 48.0, 48.0 - side)))
            .looking_at(vec3(side, -2.0, -side), Vec3::Y),
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 16f32.to_radians(),
            ..default()
        }),
        ..default()
    })
    .insert(RaycastSource::<MyRaycastSet>::new());

    // Main Base
    spawn_main_base(&mut com, &model_assets, &b);
}

fn spawn_main_base(com: &mut Commands, model_assets: &ModelAssets, b: &GameBoard) {
    let mut ecmds = com.spawn_empty();
    ecmds.insert(MainBase);
    basic_light(
        &mut ecmds,
        Color::rgb(1.0, 0.1, 1.0),
        400.0,
        4.5,
        2.0,
        vec3(0.0, 2.0, 0.0),
    );

    ecmds.insert(HookedSceneBundle {
        scene: SceneBundle {
            scene: model_assets.base.clone(),
            transform: Transform::from_translation(b.ls_to_ws_vec3(b.dest)),
            ..default()
        },
        hook: SceneHook::new(move |_entity, _cmds| {}),
    });
}

#[derive(Component)]
struct MainBase;

#[derive(Component)]
struct MainBaseDestroyed;

fn destroy_base_disable_turrets(
    mut com: Commands,
    player: Res<PlayerState>,
    main_base: Query<(Entity, &Transform), With<MainBase>>,
    model_assets: Res<ModelAssets>,
    mut turrets: Query<Entity, With<Turret>>,
) {
    if let Some((main_base_entity, main_base_trans)) = main_base.iter().next() {
        if player.health < 0.0 {
            com.entity(main_base_entity).despawn_recursive();
            com.spawn(HookedSceneBundle {
                scene: SceneBundle {
                    scene: model_assets.base_destroyed.clone(),
                    transform: *main_base_trans,
                    ..default()
                },
                hook: SceneHook::new(move |_entity, _cmds| {}),
            })
            .insert(MainBaseDestroyed);
            for entity in turrets.iter_mut() {
                com.entity(entity).insert(Disabled);
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RestartGame(bool);

fn restart_game(
    mut com: Commands,
    mut restart_game: ResMut<RestartGame>,
    mut player: ResMut<PlayerState>,
    mut b: ResMut<GameBoard>,
    model_assets: Res<ModelAssets>,
    old_base: Query<Entity, With<MainBaseDestroyed>>,
    new_base: Query<Entity, With<MainBase>>,
    enemies: Query<Entity, With<Enemy>>,
    towers: Query<Entity, With<Turret>>,
    projectiles: Query<Entity, With<Projectile>>,
    mut last_spawns: ResMut<LastSpawns>,
) {
    if **restart_game {
        **restart_game = false;
        for e in old_base.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in new_base.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in enemies.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in towers.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in projectiles.iter() {
            com.entity(e).despawn_recursive();
        }
        *b = GameBoard::default();

        let old_time_multiplier = player.time_multiplier;
        *player = PlayerState::default();
        player.time_multiplier = old_time_multiplier;

        spawn_main_base(&mut com, &model_assets, &b);

        *last_spawns = LastSpawns::default();
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
        parent.spawn(PointLightBundle {
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

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    RunLevel,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum PausedState {
    Unpaused,
    Paused,
}

pub fn game_state_asset_loading(state: Res<CurrentState<GameState>>) -> ShouldRun {
    if *state == CurrentState(GameState::AssetLoading) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn game_state_run_level_unpaused(
    state: Res<CurrentState<GameState>>,
    paused_state: Res<CurrentState<PausedState>>,
) -> ShouldRun {
    if *state == CurrentState(GameState::RunLevel)
        && *paused_state == CurrentState(PausedState::Unpaused)
    {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
