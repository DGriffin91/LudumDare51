use bevy::math::*;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::{egui::FontDefinitions, *};
use iyes_loopless::prelude::ConditionSet;

use crate::assets::GameState;
use crate::assets::ModelAssets;
use crate::board::GameBoard;
use crate::enemies::Enemy;
use crate::spawn_main_base;
use crate::turrets::Projectile;
use crate::MainBaseDestroyed;
use crate::{player::PlayerState, turrets::Turret};

pub struct GameUI;
impl Plugin for GameUI {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::RunLevel)
                    .with_system(ui_sidebar)
                    .with_system(restart_game)
                    .into(),
            )
            .add_startup_system(setup_fonts)
            .add_event::<RestartEvent>();
    }
}

const SELECTED_COLOR: Color32 = Color32::from_rgb(128, 128, 128);
const DESELECTED_COLOR: Color32 = Color32::from_rgb(48, 48, 48);

fn select_button(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
    ui.add(egui::Button::new(text).fill(if selected {
        SELECTED_COLOR
    } else {
        DESELECTED_COLOR
    }))
    .clicked()
}

fn ui_sidebar(
    mut egui_context: ResMut<EguiContext>,
    mut player: ResMut<PlayerState>,
    mut windows: ResMut<Windows>,
    mut restart: EventWriter<RestartEvent>,
) {
    let window = windows.get_primary_mut().unwrap();
    egui::SidePanel::right("right_panel")
        .resizable(false)
        .min_width(window.width() * 0.17)
        .default_width(window.width() * 0.17)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui.button("CREDITS").clicked() {
                    player.credits += 1000;
                }
                if ui.button("HEALTH").clicked() {
                    player.health += 1000.0;
                }
                if ui.button("NEXT LEVEL").clicked() {
                    player.level_time += 10.0;
                }
                ui.label(&format!("LEVEL {}", player.level as u32));
                let v = 1.0 - (player.level_time * 0.1 - player.level).fract();
                ui.label(&format!("NEXT LEVEL IN {:.2}", v * 10.0));
                ui.label("");
                ui.label(&format!("HEALTH  {:8}", (player.health * 100.0) as u32));
                ui.label(&format!("CREDITS {:8}", player.credits));
                ui.label(&format!("KILLS   {:8}", player.kills));

                if player.health > 0.0 {
                    ui.label("");
                    ui.label("TURRETS");
                    for (message, turret) in [
                        ("BLASTER", Turret::Laser),
                        ("WAVE   ", Turret::Shockwave),
                        ("LASER  ", Turret::LaserContinuous),
                    ] {
                        if select_button(
                            ui,
                            &format!("{} {:8}", message, turret.cost()),
                            player.turret_to_place == Some(turret),
                        ) {
                            player.turret_to_place = Some(turret);
                            player.sell_mode = false;
                        }
                    }
                    if select_button(ui, "SELL", player.sell_mode) {
                        player.sell_mode = !player.sell_mode;
                        if player.sell_mode {
                            player.turret_to_place = None;
                        }
                    }
                    ui.label("");
                    ui.label("UPGRADES +10%");
                    let cost = (player.blaster_upgrade.powi(2) * 10.0) as u64;
                    if ui.button(&format!("BLASTER {:8}", cost)).clicked() && player.credits > cost
                    {
                        player.blaster_upgrade *= 1.1;
                        player.credits -= cost;
                    }
                    let cost = (player.wave_upgrade.powi(2) * 10.0) as u64;
                    if ui.button(&format!("WAVE    {:8}", cost)).clicked() && player.credits > cost
                    {
                        player.wave_upgrade *= 1.1;
                        player.credits -= cost;
                    }
                    let cost = (player.laser_upgrade.powi(2) * 10.0) as u64;
                    if ui.button(&format!("LASERS  {:8}", cost)).clicked() && player.credits > cost
                    {
                        player.laser_upgrade *= 1.1;
                        player.credits -= cost;
                    }
                } else if ui.button("RESTART").clicked() {
                    restart.send(RestartEvent);
                }
            });
        });
}

pub fn setup_fonts(mut egui_context: ResMut<EguiContext>) {
    let mut fonts = FontDefinitions::default();

    for (_text_style, mut data) in fonts.font_data.iter_mut() {
        data.tweak.scale = 1.5;
        data.font =
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/FiraMono-Medium.ttf"));
    }
    egui_context.ctx_mut().set_fonts(fonts);
}

struct RestartEvent;

fn restart_game(
    mut com: Commands,
    mut restart_events: EventReader<RestartEvent>,
    mut player: ResMut<PlayerState>,
    mut b: ResMut<GameBoard>,
    model_assets: Res<ModelAssets>,
    old_base: Query<Entity, With<MainBaseDestroyed>>,
    enemies: Query<Entity, With<Enemy>>,
    towers: Query<Entity, With<Turret>>,
    projectiles: Query<Entity, With<Projectile>>,
) {
    let mut restart = false;
    for _ in restart_events.iter() {
        restart = true;
    }
    if restart {
        for e in old_base.iter() {
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
        *player = PlayerState::default();
        spawn_main_base(&mut com, &model_assets, &b);
    }
}
