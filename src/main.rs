#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use assets::{FontAssets, GameState, ModelAssets};
use bevy::{
    asset::AssetServerSettings,
    math::{ivec2, vec3},
    prelude::*,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};
use bevy_mod_picking::*;
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, Intersection, RayCastMesh, RayCastMethod, RaycastSystem,
};

use board::GameBoard;
use enemies::{destroy_enemies, spawn_enemy, Enemy, EnemyPath, Health};
use iyes_loopless::prelude::*;
use turrets::{apply_damage, Turret};
pub mod assets;
pub mod board;
pub mod enemies;
pub mod turrets;

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
    //.add_plugins(DefaultPickingPlugins)
    //.add_plugin(DebugCursorPickingPlugin)
    //.add_plugin(DebugEventsPickingPlugin)
    .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
    .add_system_to_stage(
        CoreStage::First,
        update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MyRaycastSet>),
    );

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
                .with_system(apply_damage)
                .with_system(destroy_enemies)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::RunLevel)
                .after("pre")
                .with_system(mouse_interact)
                .into(),
        );

    app.run();
}

pub fn mouse_interact(
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    intersections: Query<&Intersection<MyRaycastSet>>,
    mut b: ResMut<GameBoard>,
    buttons: Res<Input<MouseButton>>,
    mut game_cursor: Query<&mut Transform, With<GameCursor>>,
    model_assets: Res<ModelAssets>,
) {
    let mut cursor_pos = None;
    for intersection in &intersections {
        //info!(
        //    "Distance {:?}, Position {:?}",
        //    intersection.distance(),
        //    intersection.position()
        //);
        cursor_pos = intersection.position();
    }
    let cursor_pos = if let Some(cursor_pos) = cursor_pos {
        *cursor_pos
    } else {
        return;
    };
    if let Some(mut trans) = game_cursor.iter_mut().next() {
        let p = b.ls_to_ws_vec3(b.ws_vec3_to_ls(cursor_pos));
        trans.translation = p + vec3(0.0, -0.4, 0.0);
    }
    if buttons.just_pressed(MouseButton::Left) {
        if (cursor_pos.y - 0.0).abs() < 0.1 {
            let idx = b.ls_to_idx(b.ws_vec3_to_ls(cursor_pos));
            if !b.board[idx].filled {
                b.board[idx].filled = true;
                let possible_path = b.path(b.start);
                if possible_path.is_none() {
                    b.board[idx].filled = false;
                } else {
                    let unit = Turret::spawn_laser_turret(
                        &mut com,
                        b.ls_to_ws_vec3(b.idx_to_ls(idx)),
                        &model_assets,
                    );
                    b.board[idx].turret = Some(unit);
                }
            }
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        let idx = b.ls_to_idx(b.ws_vec3_to_ls(cursor_pos));
        b.destroy(&mut com, idx);
    }
    //for event in events.iter() {
    //    match event {
    //        PickingEvent::Selection(e) => info!("A selection event happened: {:?}", e),
    //        PickingEvent::Hover(e) => info!("Egads! A hover event!? {:?}", e),
    //        PickingEvent::Clicked(e) => {}
    //    }
    //}
}

pub struct MyRaycastSet;

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
            trans.translation += (next_pos - p).normalize() * time.delta_seconds() * 2.0;
        }
    }
}

#[derive(Component)]
pub struct GameCursor;

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RayCastSource<MyRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RayCastMethod::Screenspace(cursor_position);
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
        material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        ..default()
    })
    .insert_bundle(PickableBundle::default())
    .insert(Board)
    .insert(RayCastMesh::<MyRaycastSet>::default());

    com.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube {
            size: 1.0,
            ..default()
        })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, -1000.0, 0.0),
        ..default()
    })
    .insert(GameCursor);

    // light
    com.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
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
