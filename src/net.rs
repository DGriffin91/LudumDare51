use std::{net::SocketAddr, time::Duration};

use bevy::prelude::*;
use bevy_ggrs::{GGRSPlugin, SessionType};
use bytemuck::{Pod, Zeroable};
use ggrs::{Config, P2PSession, PlayerHandle, PlayerType, SessionBuilder, UdpNonBlockingSocket};
use iyes_loopless::prelude::*;
use structopt::StructOpt;

use crate::{action::ActionQueue, GameState};

#[derive(StructOpt)]
pub struct Opt {
    #[structopt(short, long, default_value = "0")]
    pub local_port: u16,
    #[structopt(short, long)]
    pub players: Vec<String>,
    #[structopt(short, long)]
    pub spectators: Vec<SocketAddr>,
}

/// You need to define a config struct to bundle all the generics of GGRS. You can safely ignore `State` and leave it as u8 for all GGRS functionality.
#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = BoxInput;
    type State = u8;
    type Address = SocketAddr;
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default, Debug)]
pub struct BoxInput {
    pub data: [u8; 3],
}

fn setup_connection(mut commands: Commands) {
    // read cmd line arguments
    let opt = Opt::from_args();
    let num_players = opt.players.len();
    assert!(num_players > 0);

    // create a GGRS session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(num_players)
        .with_max_prediction_window(1) // 1 for lockstep
        .with_input_delay(20)
        .with_disconnect_timeout(Duration::from_secs(20));

    // add players
    for (i, player_addr) in opt.players.iter().enumerate() {
        // local player
        if player_addr == "localhost" {
            sess_build = sess_build.add_player(PlayerType::Local, i).unwrap();
        } else {
            // remote players
            let remote_addr: SocketAddr = player_addr.parse().unwrap();
            sess_build = sess_build
                .add_player(PlayerType::Remote(remote_addr), i)
                .unwrap();
        }
    }

    // optionally, add spectators
    for (i, spec_addr) in opt.spectators.iter().enumerate() {
        sess_build = sess_build
            .add_player(PlayerType::Spectator(*spec_addr), num_players + i)
            .unwrap();
    }

    // start the GGRS session
    let socket = UdpNonBlockingSocket::bind_to_port(opt.local_port).unwrap();

    let sess = sess_build.start_p2p_session(socket).unwrap();

    commands.insert_resource(sess);
    commands.insert_resource(opt);
    commands.insert_resource(NextState(GameState::RunLevel));
}

pub(crate) fn setup_ggrs(app: &mut bevy::prelude::App, rollback_stage: SystemStage) {
    app.add_enter_system(GameState::Connecting, setup_connection)
        .add_enter_system(GameState::RunLevel, setup_session)
        //print some network stats - not part of the rollback schedule as it does not need to be rolled back
        .insert_resource(NetworkStatsTimer(Timer::from_seconds(2.0, true)));

    GGRSPlugin::<GGRSConfig>::new()
        // define frequency of rollback game logic update
        .with_update_frequency(120)
        // define system that returns inputs given a player handle, so GGRS can send the inputs around
        .with_input_system(input)
        // register types of components AND resources you want to be rolled back
        //.register_rollback_type::<Transform>()
        // these systems will be executed as part of the advance frame update
        .with_rollback_schedule(Schedule::default().with_stage("my_fixed_update", rollback_stage))
        // make it happen in the bevy app
        .build(app);

    app.add_system(print_network_stats_system);
}

pub fn input(_handle: In<PlayerHandle>, action_queue: Res<ActionQueue>) -> BoxInput {
    let mut input = BoxInput::default();

    //TODO support multiple actions
    if let Some(action) = action_queue.tx.last() {
        input.data = action.to_bytes();
    }

    input
}

pub fn setup_session(mut commands: Commands) {
    // add your GGRS session
    commands.insert_resource(SessionType::P2PSession);
    // register a resource that will be rolled back
    //commands.insert_resource(FrameCount { frame: 0 });
}

struct NetworkStatsTimer(Timer);
fn print_network_stats_system(
    time: Res<Time>,
    mut timer: ResMut<NetworkStatsTimer>,
    p2p_session: Option<Res<P2PSession<GGRSConfig>>>,
) {
    // print only when timer runs out
    if timer.0.tick(time.delta()).just_finished() {
        if let Some(sess) = p2p_session {
            let num_players = sess.num_players() as usize;
            for i in 0..num_players {
                if let Ok(stats) = sess.network_stats(i) {
                    println!("NetworkStats for player {}: {:?}", i, stats);
                }
            }
        }
    }
}
