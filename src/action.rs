use bevy::{math::*, prelude::*};

use crate::{
    assets::ModelAssets, board::GameBoard, player::PlayerState, turrets::Turret, ui::Preferences,
    GameTime, RestartGame,
};

pub fn process_actions(
    mut com: Commands,
    mut action_queue: ResMut<ActionQueue>,
    mut player: ResMut<PlayerState>,
    mut time: ResMut<GameTime>,
    mut restart: ResMut<RestartGame>,
    model_assets: Res<ModelAssets>,
    mut b: ResMut<GameBoard>,
    pref: Res<Preferences>,
) {
    if action_queue.0.len() > 1 {
        // See if this ever happens
        println!("action_queue over 1: {}", action_queue.0.len())
    }

    #[allow(unused_assignments)]
    let mut debug_build = false;

    #[cfg(debug_assertions)]
    {
        debug_build = true;
    }

    for action in action_queue.0.iter() {
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
                time.time_multiplier = (time.time_multiplier - 0.1).max(0.0);
            }
            Action::GameSpeedInc => {
                time.time_multiplier = (time.time_multiplier + 0.1).min(10.0);
            }
            Action::GamePause => {
                time.pause = !time.pause;
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

    action_queue.0 = Vec::new(); // Clear action queue
}

#[derive(Deref, DerefMut, Default)]
pub struct ActionQueue(pub Vec<Action>);

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
