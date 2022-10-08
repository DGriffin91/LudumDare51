use bevy::{math::*, prelude::*};
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, Intersection, RayCastMethod, RayCastSource, RaycastSystem,
};
use iyes_loopless::prelude::*;

use crate::{
    action::{Action, ActionQueue},
    assets::GameState,
    board::GameBoard,
    turrets::Turret,
    GameTime,
};

pub struct GameSettings {
    pub rolling_enemy_health: f32,
    pub rolling_enemy_speed: f32,
    pub rolling_enemy_spawn_speed: f32,
    pub rolling_enemy_max_spawn_speed: f32,
    //
    pub rolling_enemy_2_health: f32,
    pub rolling_enemy_2_speed: f32,
    pub rolling_enemy_2_spawn_speed: f32,
    pub rolling_enemy_2_max_spawn_speed: f32,
    //
    pub flying_enemy_health: f32,
    pub flying_enemy_speed: f32,
    pub flying_enemy_spawn_speed: f32,
    pub flying_enemy_max_spawn_speed: f32,
    //
    pub credits_for_kill: u64,
}

pub struct PlayerState {
    pub credits: u64,
    pub turret_to_place: Option<Turret>,
    pub kills: u64,
    pub health: f32,
    pub sell_mode: bool,
    pub blaster_upgrade: f32,
    pub laser_upgrade: f32,
    pub wave_upgrade: f32,
    pub level_time: f32,
    pub level: f32,
}

pub const GAMESETTINGS: GameSettings = GameSettings {
    rolling_enemy_health: 0.5,
    rolling_enemy_speed: 0.6,
    rolling_enemy_spawn_speed: 3.0,
    rolling_enemy_max_spawn_speed: 1.2,
    //
    rolling_enemy_2_health: 0.28,
    rolling_enemy_2_speed: 1.2,
    rolling_enemy_2_spawn_speed: 3.5,
    rolling_enemy_2_max_spawn_speed: 1.1,
    //
    flying_enemy_health: 0.08,
    flying_enemy_speed: 2.0,
    flying_enemy_spawn_speed: 3.0,
    flying_enemy_max_spawn_speed: 0.05,
    //
    credits_for_kill: 25,
};

impl PlayerState {
    pub fn enemy_speed_boost(&self) -> f32 {
        self.level.powf(0.4) * 0.1
    }

    pub fn spawn_rate_cut(&self) -> f32 {
        self.level.powf(0.4) * 0.3
    }

    pub fn enemy_health_mult(&self) -> f32 {
        if self.level < 50.0 {
            (self.level.powf(1.32) + 1.0) / 2.0
        } else {
            (self.level.powf(1.32) + 1.0) / (2.0 - (self.level - 50.0) * 0.5).max(1.0)
        }
    }

    pub fn blaster_upgrade_cost(&self) -> u64 {
        (self.blaster_upgrade.powi(2) * 25.0) as u64
    }

    pub fn wave_upgrade_cost(&self) -> u64 {
        (self.wave_upgrade.powi(2) * 25.0) as u64
    }

    pub fn laser_upgrade_cost(&self) -> u64 {
        (self.laser_upgrade.powi(2) * 25.0) as u64
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            credits: 500,
            turret_to_place: None,
            kills: 0,
            health: 1.0,
            sell_mode: false,
            blaster_upgrade: 1.0,
            laser_upgrade: 1.0,
            wave_upgrade: 1.0,
            level_time: 0.0,
            level: 0.0,
        }
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
            .insert_resource(PlayerState::default())
            .add_enter_system(GameState::RunLevel, setup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::RunLevel)
                    .with_system(mouse_interact)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::RunLevel)
                    .run_if(player_alive)
                    .with_system(set_level)
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

pub fn player_alive(player: Res<PlayerState>) -> bool {
    player.health > 0.0
}

fn set_level(time: ResMut<GameTime>, mut player: ResMut<PlayerState>) {
    player.level_time += time.delta_seconds;
    player.level = (player.level_time / 10.0).floor();
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
    intersections: Query<&Intersection<MyRaycastSet>>,
    b: Res<GameBoard>,
    buttons: Res<Input<MouseButton>>,
    mut game_cursor: Query<&mut Transform, With<GameCursor>>,
    player: Res<PlayerState>,
    mut action_queue: ResMut<ActionQueue>,
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
        let ls_p = b.idx_to_ls(idx);
        if player.sell_mode {
            action_queue.push(Action::SellTurret(ls_p.x as u8, ls_p.y as u8));
        } else if let Some(selected_turret) = player.turret_to_place {
            match selected_turret {
                Turret::Blaster => {
                    action_queue.push(Action::BlasterPlace(ls_p.x as u8, ls_p.y as u8));
                }
                Turret::Laser => {
                    action_queue.push(Action::LaserPlace(ls_p.x as u8, ls_p.y as u8));
                }
                Turret::Wave => {
                    action_queue.push(Action::WavePlace(ls_p.x as u8, ls_p.y as u8));
                }
            };
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
