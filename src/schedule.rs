use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastSystem};
use bevy_system_graph::SystemGraph;
use iyes_loopless::prelude::*;

use crate::{
    action::*, enemies::*, game_state_run_level_unpaused, player::*, restart_game, turrets::*,
    GameState,
};

pub const TIMESTEP_MILLI: u64 = 16;
pub const TIMESTEP: f32 = 0.016;
pub const TIMESTEP_SEC_F64: f64 = 0.016;

pub(crate) fn setup_schedule(app: &mut bevy::prelude::App) -> SystemStage {
    let mut fixed_update_stage = SystemStage::parallel();

    app.add_system_set(
        ConditionSet::new()
            .run_in_state(GameState::RunLevel)
            .with_system(mouse_interact)
            .with_system(playback_actions)
            .into(),
    );

    app.add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .insert_resource(PlayerState::default())
        .add_enter_system(GameState::RunLevel, setup_player)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MyRaycastSet>),
        );

    fixed_update_stage.add_system_set(
        Into::<SystemSet>::into(SystemGraph::new().root(set_level).graph())
            .with_run_criteria(game_state_run_level_unpaused)
            .label("STEP PLAYER")
            .before("STEP ENEMIES"),
    );

    fixed_update_stage.add_system_set(
        Into::<SystemSet>::into(
            SystemGraph::new()
                .root(destroy_enemies)
                .then(spawn_rolling_enemy)
                .then(spawn_rolling_enemy2)
                .then(spawn_flying_enemy)
                .then(update_enemy_paths)
                .then(update_board_has_enemy)
                .then(move_enemy_along_path)
                .then(move_flying_enemy)
                .then(check_enemy_at_dest)
                .then(check_flying_enemy_at_dest)
                .then(update_enemy_postgame_paths)
                .then(update_flying_enemy_postgame_dest)
                .graph(),
        )
        .with_run_criteria(game_state_run_level_unpaused)
        .label("STEP ENEMIES"),
    );

    fixed_update_stage.add_system_set(
        Into::<SystemSet>::into(
            SystemGraph::new()
                .root(progress_projectiles)
                .then(progress_explosions)
                .then(bobble_shockwave_spheres)
                .then(position_caps)
                .then(reset_turret_gfx)
                .then(turret_fire)
                .then(blaster_point_at_enemy)
                .graph(),
        )
        .with_run_criteria(game_state_run_level_unpaused)
        .label("STEP TURRET")
        .after("STEP ENEMIES"),
    );

    app.insert_resource(ActionQueue::default());
    app.insert_resource(GameRecorder::default());
    fixed_update_stage.add_system_set(
        ConditionSet::new()
            .run_in_state(GameState::RunLevel)
            .label("STEP ACTION")
            .after("STEP TURRET")
            .with_system(process_actions)
            .into(),
    );

    fixed_update_stage.add_system_set(
        ConditionSet::new()
            .run_in_state(GameState::RunLevel)
            .label("STEP RESTART GAME")
            .after("STEP TURRET")
            .with_system(restart_game)
            .into(),
    );

    fixed_update_stage
}
