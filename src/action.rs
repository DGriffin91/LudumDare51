use std::time::Duration;

use bevy::{math::*, prelude::*};

use ggrs::InputStatus;
use iyes_loopless::{
    prelude::FixedTimestepInfo,
    state::{CurrentState, NextState},
};
use rkyv::{Archive, Deserialize, Serialize};

use bytecheck::CheckBytes;

use crate::{
    assets::ModelAssets, board::GameBoard, net::BoxInput, player::PlayerState,
    schedule::TIMESTEP_MILLI, turrets::Turret, ui::Preferences, PausedState, RestartGame,
};

pub fn playback_actions(
    player: Res<PlayerState>,
    mut game_recorder: ResMut<GameRecorder>,
    mut action_queue: ResMut<ActionQueue>,
) {
    if game_recorder.play {
        while let Some((step, rec_actions)) = game_recorder.actions.0.get(game_recorder.play_head) {
            if *step as u64 == player.step {
                action_queue.tx.push(Action::from_bytes(*rec_actions))
            } else {
                break;
            }
            game_recorder.play_head += 1;
        }
    }
}

pub fn process_actions(
    mut com: Commands,
    mut action_queue: ResMut<ActionQueue>,
    mut player: ResMut<PlayerState>,
    mut restart: ResMut<RestartGame>,
    model_assets: Res<ModelAssets>,
    mut b: ResMut<GameBoard>,
    pref: Res<Preferences>,
    mut time_step_info: Option<ResMut<FixedTimestepInfo>>,
    paused_state: Res<CurrentState<PausedState>>,
    mut game_recorder: ResMut<GameRecorder>,
    net_rx: Option<Res<Vec<(BoxInput, InputStatus)>>>,
) {
    if let Some(net_rx) = net_rx {
        //for net_player in net_players.iter_mut() {
        //let input = net_rx[net_player.handle as usize].0;
        for (input, _status) in net_rx.iter() {
            action_queue.rx.push(Action::from_bytes(input.data));
        }
        //}
    } else {
        // this is single player, just pass actions from rx to tx
        action_queue.rx = action_queue.tx.clone();
    }

    #[allow(unused_assignments, unused_mut)]
    let mut debug_build = false;

    #[cfg(debug_assertions)]
    {
        debug_build = true;
    }

    for action in action_queue.rx.iter() {
        let mut place_turret = None;
        match action {
            Action::Empty => continue,
            Action::BlasterUpgrade => {
                let cost = player.blaster_upgrade_cost();
                if player.credits > cost {
                    player.blaster_upgrade *= 1.05;
                    player.credits -= cost;
                }
            }
            Action::WaveUpgrade => {
                let cost = player.wave_upgrade_cost();
                if player.credits > cost {
                    player.wave_upgrade *= 1.05;
                    player.credits -= cost;
                }
            }
            Action::LaserUpgrade => {
                let cost = player.laser_upgrade_cost();
                if player.credits > cost {
                    player.laser_upgrade *= 1.05;
                    player.credits -= cost;
                }
            }
            Action::BlasterPlace(x, y) => {
                place_turret = Some((Turret::Blaster, x, y));
            }
            Action::WavePlace(x, y) => {
                place_turret = Some((Turret::Wave, x, y));
            }
            Action::LaserPlace(x, y) => {
                place_turret = Some((Turret::Laser, x, y));
            }
            Action::SellTurret(x, y) => {
                let idx = b.ls_to_idx(ivec2(*x as i32, *y as i32));
                let turret = b.destroy(&mut com, idx);
                if let Some(turret) = turret {
                    // Player gets back 50% of cost when selling
                    player.credits += turret.cost() / 2;
                }
            }
            Action::GameSpeedDec => {
                if let Some(ref mut time_step_info) = time_step_info {
                    player.time_multiplier = (player.time_multiplier - 0.1).max(0.1);
                    time_step_info.step = Duration::from_millis(
                        (TIMESTEP_MILLI as f64 / player.time_multiplier) as u64,
                    )
                }
            }
            Action::GameSpeedInc => {
                if let Some(ref mut time_step_info) = time_step_info {
                    player.time_multiplier = (player.time_multiplier + 0.1).min(10.0);
                    time_step_info.step = Duration::from_millis(
                        (TIMESTEP_MILLI as f64 / player.time_multiplier) as u64,
                    )
                }
            }
            Action::GamePause => {
                if *paused_state == CurrentState(PausedState::Paused) {
                    com.insert_resource(NextState(PausedState::Unpaused));
                } else {
                    com.insert_resource(NextState(PausedState::Paused));
                }
            }
            Action::RestartGame => {
                **restart = true;
            }
            Action::CheatCredits => {
                if debug_build {
                    player.credits += 1000;
                }
            }
            Action::CheatHealth => {
                if debug_build {
                    player.health += 1000.0;
                }
            }
            Action::CheatLevel => {
                if debug_build {
                    player.level_time += 10.0;
                }
            }
        }
        if let Some((turret, x, y)) = place_turret {
            let idx = b.ls_to_idx(ivec2(*x as i32, *y as i32));
            if !b.board[idx].filled && idx != b.ls_to_idx(b.start) {
                b.board[idx].filled = true; //Just temp fill so we can check
                let possible_path = b.path(b.start, b.dest);
                b.board[idx].filled = false; //Undo temp fill
                if possible_path.is_some() {
                    let cost = turret.cost();
                    if player.credits >= cost {
                        player.credits -= cost;
                        let idx = b.ls_to_idx(ivec2(*x as i32, *y as i32));
                        let pos = b.ls_to_ws_vec3(b.idx_to_ls(idx));
                        b.board[idx].turret = Some(match turret {
                            Turret::Blaster => {
                                Turret::spawn_blaster_turret(&mut com, pos, &model_assets, &pref)
                            }
                            Turret::Laser => Turret::spawn_laser_continuous_turret(
                                &mut com,
                                pos,
                                &model_assets,
                                &pref,
                            ),
                            Turret::Wave => {
                                Turret::spawn_shockwave_turret(&mut com, pos, &model_assets, &pref)
                            }
                        });

                        b.board[idx].filled = true;
                    }
                }
            }
        }
    }

    if !game_recorder.disable_rec {
        action_queue.rx.retain_mut(|action| {
            !matches!(
                action,
                Action::Empty
                    | Action::GameSpeedDec
                    | Action::GameSpeedInc
                    | Action::GamePause
                    | Action::RestartGame
            )
        });

        for action in action_queue.rx.iter() {
            game_recorder
                .actions
                .0
                .push((player.step as u32, action.to_bytes()));
        }
    }

    action_queue.tx = Vec::new(); // Clear action queue
    action_queue.rx = Vec::new(); // Clear action queue
}

#[derive(Archive, Deserialize, Serialize, Clone, Eq, PartialEq, Default, Debug)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes))]
pub struct ActionRecording(Vec<(u32, [u8; 3])>);

#[derive(Default)]
pub struct GameRecorder {
    pub actions: ActionRecording,
    pub disable_rec: bool,
    pub play: bool,
    pub play_head: usize,
}

#[derive(Default)]
pub struct ActionQueue {
    pub tx: Vec<Action>,
    pub rx: Vec<Action>,
}

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Eq, PartialEq, Debug)]
#[archive_attr(derive(CheckBytes))]
pub enum Action {
    Empty,
    BlasterUpgrade,
    WaveUpgrade,
    LaserUpgrade,
    BlasterPlace(u8, u8),
    WavePlace(u8, u8),
    LaserPlace(u8, u8),
    SellTurret(u8, u8),
    GameSpeedDec,
    GameSpeedInc,
    GamePause,
    RestartGame,
    CheatCredits,
    CheatHealth,
    CheatLevel,
}

impl Action {
    #[rustfmt::skip]
    pub fn to_bytes(&self) -> [u8; 3] {
        match self {
            Action::Empty                        => [ 0,  0,  0],
            Action::BlasterUpgrade               => [ 1,  0,  0],
            Action::WaveUpgrade                  => [ 2,  0,  0],
            Action::LaserUpgrade                 => [ 3,  0,  0],
            Action::BlasterPlace(x, y) => [ 4, *x, *y],
            Action::WavePlace(x, y)    => [ 5, *x, *y],
            Action::LaserPlace(x, y)   => [ 6, *x, *y],
            Action::SellTurret(x, y)   => [ 7, *x, *y],
            Action::GameSpeedDec                 => [ 8,  0,  0],
            Action::GameSpeedInc                 => [ 9,  0,  0],
            Action::GamePause                    => [10,  0,  0],
            Action::RestartGame                  => [11,  0,  0],
            Action::CheatCredits                 => [12,  0,  0],
            Action::CheatHealth                  => [13,  0,  0],
            Action::CheatLevel                   => [14,  0,  0],
        }
    }

    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        let x = bytes[1];
        let y = bytes[2];
        match bytes[0] {
            0 => Action::Empty,
            1 => Action::BlasterUpgrade,
            2 => Action::WaveUpgrade,
            3 => Action::LaserUpgrade,
            4 => Action::BlasterPlace(x, y),
            5 => Action::WavePlace(x, y),
            6 => Action::LaserPlace(x, y),
            7 => Action::SellTurret(x, y),
            8 => Action::GameSpeedDec,
            9 => Action::GameSpeedInc,
            10 => Action::GamePause,
            11 => Action::RestartGame,
            12 => Action::CheatCredits,
            13 => Action::CheatHealth,
            14 => Action::CheatLevel,
            _ => Action::Empty,
        }
    }
}
