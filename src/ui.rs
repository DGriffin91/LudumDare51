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
use crate::GameTime;
use crate::MainBase;
use crate::MainBaseDestroyed;
use crate::{player::PlayerState, turrets::Turret};

pub struct GameUI;
impl Plugin for GameUI {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(Preferences::default())
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

const SELECTED_COLOR: Color32 = Color32::from_rgb(94 / 3, 255 / 3, 169 / 3);
const DESELECTED_COLOR: Color32 = Color32::from_rgb(94 / 10, 255 / 10, 169 / 10);

fn select_button(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
    ui.add(egui::Button::new(text).fill(if selected {
        SELECTED_COLOR
    } else {
        DESELECTED_COLOR
    }))
    .clicked()
}

fn ui_sidebar(
    mut time: ResMut<GameTime>,
    mut egui_context: ResMut<EguiContext>,
    mut player: ResMut<PlayerState>,
    mut windows: ResMut<Windows>,
    mut restart: EventWriter<RestartEvent>,
    mut pref: ResMut<Preferences>,
) {
    let window = windows.get_primary_mut().unwrap();
    let my_frame = egui::containers::Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 64),
        stroke: egui::Stroke::new(0.0, Color32::BLACK),
        ..default()
    };

    egui::SidePanel::right("right_panel")
        .frame(my_frame)
        .resizable(false)
        .min_width(window.width() * 0.17)
        .default_width(window.width() * 0.17)
        .show(egui_context.ctx_mut(), |ui| {
            let mut style = ui.style_mut();
            style.visuals.override_text_color = Some(Color32::from_rgb(94, 255, 169));
            style.visuals.widgets.active.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.inactive.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.open.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.hovered.bg_fill = SELECTED_COLOR;
            ui.vertical_centered_justified(|ui| {
                #[cfg(debug_assertions)]
                {
                    if ui.button("CREDITS").clicked() {
                        player.credits += 1000;
                    }
                    if ui.button("HEALTH").clicked() {
                        player.health += 1000.0;
                    }
                    if ui.button("NEXT LEVEL").clicked() {
                        player.level_time += 10.0;
                    }
                }
                let v = 1.0 - (player.level_time * 0.1 - player.level).fract();
                ui.label(&format!(
                    "LEVEL {}   NEXT {:.2}",
                    player.level as u32,
                    v * 10.0
                ));
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
                    ui.label("UPGRADES +5%");
                    let cost = (player.blaster_upgrade.powi(2) * 25.0) as u64;
                    if ui.button(&format!("BLASTER {:8}", cost)).clicked() && player.credits > cost
                    {
                        player.blaster_upgrade *= 1.05;
                        player.credits -= cost;
                    }
                    let cost = (player.wave_upgrade.powi(2) * 25.0) as u64;
                    if ui.button(&format!("WAVE    {:8}", cost)).clicked() && player.credits > cost
                    {
                        player.wave_upgrade *= 1.05;
                        player.credits -= cost;
                    }
                    let cost = (player.laser_upgrade.powi(2) * 25.0) as u64;
                    if ui.button(&format!("LASERS  {:8}", cost)).clicked() && player.credits > cost
                    {
                        player.laser_upgrade *= 1.05;
                        player.credits -= cost;
                    }
                    ui.label("");

                    ui.label(&format!("GAME SPEED {:.2}", time.time_multiplier));
                    ui.horizontal(|ui| {
                        if ui.button(" -- ").clicked() {
                            time.time_multiplier = (time.time_multiplier - 0.1).max(0.0);
                        }
                        if ui.button(" ++ ").clicked() {
                            time.time_multiplier += 0.1;
                        }
                        if ui.button("PAUSE").clicked() {
                            time.pause = !time.pause;
                        }
                    });
                }
                if ui
                    .checkbox(&mut pref.less_lights, "REDUCE LIGHTS")
                    .changed()
                {
                    if pref.less_lights {
                        pref.light_r = 0.6;
                    } else {
                        pref.light_r = 1.0;
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button(" -- ").clicked() {
                        pref.sfx = (pref.sfx - 0.1).max(0.0);
                    }
                    if ui.button(" ++ ").clicked() {
                        pref.sfx = (pref.sfx + 0.1).min(3.0);
                    }
                    ui.label(&format!("SFX {:.1}", pref.sfx));
                });
                ui.horizontal(|ui| {
                    if ui.button(" -- ").clicked() {
                        pref.sfx = (pref.sfx - 0.1).max(0.0);
                    }
                    if ui.button(" ++ ").clicked() {
                        pref.sfx = (pref.sfx + 0.1).min(3.0);
                    }
                    ui.label(&format!("MUSIC {:.1}", pref.sfx));
                });
                ui.label("");
                if ui.button("RESTART GAME").clicked() {
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
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/ShareTechMono-Regular.ttf"));
    }
    egui_context.ctx_mut().set_fonts(fonts);
}

pub struct RestartEvent;

fn restart_game(
    mut com: Commands,
    mut restart_events: EventReader<RestartEvent>,
    mut player: ResMut<PlayerState>,
    mut b: ResMut<GameBoard>,
    model_assets: Res<ModelAssets>,
    old_base: Query<Entity, With<MainBaseDestroyed>>,
    new_base: Query<Entity, With<MainBase>>,
    enemies: Query<Entity, With<Enemy>>,
    towers: Query<Entity, With<Turret>>,
    projectiles: Query<Entity, With<Projectile>>,
    mut time: ResMut<GameTime>,
) {
    let mut restart = false;
    for _ in restart_events.iter() {
        restart = true;
    }
    if restart {
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
        *player = PlayerState::default();
        spawn_main_base(&mut com, &model_assets, &b);
        *time = GameTime::default();
    }
}

pub struct Preferences {
    pub less_lights: bool,
    pub light_r: f32, //light range mult
    pub sfx: f64,
    pub music: f64,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            less_lights: false,
            light_r: 1.0,
            sfx: 1.0,
            music: 1.0,
        }
    }
}
