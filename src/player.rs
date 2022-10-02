use bevy::{math::*, prelude::*};
use bevy_mod_picking::RayCastSource;
use bevy_mod_raycast::{DefaultRaycastingPlugin, Intersection, RayCastMethod, RaycastSystem};
use iyes_loopless::prelude::*;

use crate::{
    assets::{GameState, ModelAssets},
    board::GameBoard,
    turrets::Turret,
};

pub struct PlayerState {
    pub credits: u64,
    pub turret_to_place: Option<Turret>,
    pub kills: u64,
    pub health: f32,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
            .insert_resource(PlayerState {
                credits: 5000,
                turret_to_place: None,
                kills: 0,
                health: 1.0,
            })
            .add_enter_system(GameState::RunLevel, setup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::RunLevel)
                    .after("pre")
                    .with_system(mouse_interact)
                    .into(),
            )
            .add_system_to_stage(
                CoreStage::First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MyRaycastSet>),
            );
        //.add_plugins(DefaultPickingPlugins)
        //.add_plugin(DebugCursorPickingPlugin)
        //.add_plugin(DebugEventsPickingPlugin)
    }
}

fn setup(
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    com.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: Color::rgb(0.2, 0.2, 0.2),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -1000.0, 0.0),
        ..default()
    })
    .insert(GameCursor);
}

pub struct MyRaycastSet;

pub fn mouse_interact(
    mut com: Commands,
    intersections: Query<&Intersection<MyRaycastSet>>,
    mut b: ResMut<GameBoard>,
    buttons: Res<Input<MouseButton>>,
    mut game_cursor: Query<&mut Transform, With<GameCursor>>,
    model_assets: Res<ModelAssets>,
    mut player: ResMut<PlayerState>,
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
    if buttons.just_pressed(MouseButton::Left) && (cursor_pos.y - 0.0).abs() < 0.1 {
        let idx = b.ls_to_idx(b.ws_vec3_to_ls(cursor_pos));
        if !b.board[idx].filled {
            b.board[idx].filled = true; //Just temp fill so we can check
            let possible_path = b.path(b.start);
            b.board[idx].filled = false; //Undo temp fill
            if possible_path.is_some() {
                if let Some(selected_turret) = player.turret_to_place {
                    let cost = selected_turret.cost();
                    if player.credits >= cost {
                        player.credits -= cost;
                        let pos = b.ls_to_ws_vec3(b.idx_to_ls(idx));
                        b.board[idx].turret = Some(match selected_turret {
                            Turret::Laser => {
                                Turret::spawn_laser_turret(&mut com, pos, &model_assets)
                            }
                            Turret::LaserContinuous => {
                                Turret::spawn_laser_continuous_turret(&mut com, pos, &model_assets)
                            }
                            Turret::Shockwave => {
                                Turret::spawn_shockwave_turret(&mut com, pos, &model_assets)
                            }
                        });
                        b.board[idx].filled = true;
                    }
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
