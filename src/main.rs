#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    math::{ivec2, vec3},
    prelude::*,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_mod_picking::*;
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, Intersection, RayCastMesh, RayCastMethod, RaycastSystem,
};

use pathfinding::prelude::*;

fn main() {
    let mut app = App::new();
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
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(DefaultPlugins)
    //.add_plugins(DefaultPickingPlugins)
    //.add_plugin(DebugCursorPickingPlugin)
    //.add_plugin(DebugEventsPickingPlugin)
    .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
    .add_startup_system(setup)
    .add_system_to_stage(CoreStage::PostUpdate, mouse_interact)
    .add_system_to_stage(
        CoreStage::First,
        update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MyRaycastSet>),
    );

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    app.add_system(spawn_enemy)
        .add_system(update_enemy_paths)
        .add_system(move_enemy_along_path);

    app.run();
}

#[derive(Copy, Clone)]
pub struct Cell {
    filled: bool,
    tower_entity: Option<Entity>,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            filled: false,
            tower_entity: None,
        }
    }
}

pub struct GameBoard {
    size: [usize; 2],
    position: IVec2,
    board: Vec<Cell>,
    start: IVec2,
    dest: IVec2,
}

impl GameBoard {
    fn new(position: IVec2, size: [usize; 2], start: IVec2, dest: IVec2) -> GameBoard {
        let board = vec![Cell::default(); size[0] * size[1]];
        GameBoard {
            size,
            position,
            board,
            start,
            dest,
        }
    }

    #[inline(always)]
    fn ls_to_ws(&self, ls: IVec2) -> IVec2 {
        ls + self.position
    }

    #[inline(always)]
    fn ws_to_ls(&self, ws: IVec2) -> IVec2 {
        ws - self.position
    }

    #[inline(always)]
    fn ls_to_idx(&self, ls: IVec2) -> usize {
        let x = (ls.x as usize).clamp(0, self.size[0] - 1);
        let y = (ls.y as usize).clamp(0, self.size[1] - 1);
        x + y * self.size[1]
    }

    #[inline(always)]
    pub fn idx_to_ls(&self, idx: usize) -> IVec2 {
        let x = idx % self.size[0];
        let y = (idx / self.size[0]) % self.size[1];
        let pos = ivec2(x as i32, y as i32);
        pos
    }

    pub fn path(&self, start: IVec2) -> Option<(Vec<IVec2>, u32)> {
        let a = astar(
            &start,
            |p| self.successors(*p),
            |p| {
                let a = (self.dest - *p).abs();
                (a.x + a.y) as u32
            },
            |p| *p == self.dest,
        );
        a
    }

    #[inline(always)]
    pub fn successors(&self, ls: IVec2) -> Vec<(IVec2, u32)> {
        let mut s = Vec::new();
        for offset in [
            //ivec2(0, 0),
            ivec2(-1, 0),
            ivec2(1, 0),
            ivec2(0, -1),
            ivec2(0, 1),
        ] {
            let potential_pos = ls + offset;
            if potential_pos.clamp(IVec2::ZERO, ivec2(self.size[0] as i32, self.size[1] as i32))
                != potential_pos
            {
                continue;
            }
            if self.board[self.ls_to_idx(potential_pos)].filled {
                continue;
            }
            s.push((potential_pos, 1))
        }

        s
    }
    fn ws_vec3_to_ls(&self, ws: Vec3) -> IVec2 {
        self.ws_to_ls(vec3_to_ivec2(ws))
    }
    fn ls_to_ws_vec3(&self, ls: IVec2) -> Vec3 {
        ivec2_to_vec3(self.ls_to_ws(ls)) + vec3(0.5, 0.0, 0.5)
    }
}

fn vec3_to_ivec2(p: Vec3) -> IVec2 {
    ivec2(p.x.floor() as i32, p.z.floor() as i32)
}

fn ivec2_to_vec3(p: IVec2) -> Vec3 {
    vec3(p.x as f32, 0.0, p.y as f32)
}

pub fn mouse_interact(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    intersections: Query<&Intersection<MyRaycastSet>>,
    mut b: ResMut<GameBoard>,
    buttons: Res<Input<MouseButton>>,
    mut game_cursor: Query<&mut Transform, With<GameCursor>>,
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
            let idx = b.ls_to_idx(b.ws_to_ls(vec3_to_ivec2(cursor_pos)));
            if !b.board[idx].filled {
                b.board[idx].filled = true;
                let possible_path = b.path(b.start);
                if possible_path.is_none() {
                    b.board[idx].filled = false;
                } else {
                    let entity = commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: 0.5,
                                ..default()
                            })),
                            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                            transform: Transform::from_translation(
                                b.ls_to_ws_vec3(b.idx_to_ls(idx)) + vec3(0.0, 0.5, 0.0),
                            ),
                            ..default()
                        })
                        .insert_bundle(PickableBundle::default())
                        .insert(RayCastMesh::<MyRaycastSet>::default())
                        .id();
                    b.board[idx].tower_entity = Some(entity);
                }
            }
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        let idx = b.ls_to_idx(b.ws_to_ls(vec3_to_ivec2(cursor_pos)));
        if b.board[idx].filled {
            if let Some(entity) = b.board[idx].tower_entity {
                commands.entity(entity).despawn();
                b.board[idx].filled = false;
            }
        }
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
pub struct Enemy {
    path: Option<(Vec<IVec2>, u32)>,
}

#[derive(Component)]
struct PathInd;

fn update_enemy_paths(
    mut commands: Commands,
    b: Res<GameBoard>,
    mut enemies: Query<(&Transform, &mut Enemy)>,
    path_ind: Query<Entity, With<PathInd>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (trans, mut enemy) in enemies.iter_mut() {
        enemy.path = b.path(b.ws_to_ls(vec3_to_ivec2(trans.translation)));
    }
    for entity in &path_ind {
        commands.entity(entity).despawn();
    }
    if let Some((_, enemy)) = enemies.iter().next() {
        for path in &enemy.path {
            for p in &path.0 {
                commands
                    .spawn_bundle(PbrBundle {
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
    mut enemies: Query<(&mut Transform, &mut Enemy)>,
) {
    for (mut trans, enemy) in enemies.iter_mut() {
        if let Some(path) = &enemy.path {
            let p = trans.translation;
            let a = b.ls_to_ws_vec3(path.0[1]) + vec3(0.0, 0.5, 0.0);
            let next_pos = a;
            trans.translation += (next_pos - p).normalize() * time.delta_seconds() * 2.0;
        }
    }
}

fn spawn_enemy(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_spawn: Local<f32>,
    b: Res<GameBoard>,
) {
    let since_startup = time.seconds_since_startup() as f32;
    if since_startup - *last_spawn > 10.0 {
        *last_spawn = since_startup;
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.5,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
                transform: Transform::from_translation(
                    b.ls_to_ws_vec3(b.start) + vec3(0.0, 0.5, 0.0),
                ),
                ..default()
            })
            .insert(Enemy { path: None });
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
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GameBoard::new(
        ivec2(-12, -12),
        [24, 24],
        ivec2(0, 0),
        ivec2(22, 22),
    ));
    //commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 24.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(Board)
        .insert(RayCastMesh::<MyRaycastSet>::default());

    commands
        .spawn_bundle(PbrBundle {
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
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 24.0, 18.0)
                .looking_at(vec3(0.0, 0.0, 3.0), Vec3::Y),
            ..default()
        })
        .insert_bundle(PickingCameraBundle::default())
        .insert(RayCastSource::<MyRaycastSet>::new());
}
