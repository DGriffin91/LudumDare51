use bevy::math::*;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::{egui::FontDefinitions, *};
use iyes_loopless::prelude::ConditionSet;
use lz4_flex::compress_prepend_size;
use lz4_flex::decompress_size_prepended;
use rkyv::Deserialize;

use crate::action::Action;
use crate::action::ActionQueue;
use crate::action::ActionRecording;
use crate::action::GameRecorder;
use crate::audio::AudioEvents;
use crate::audio::MUSIC_LEVEL_CHANGED;
use crate::audio::SFX_LEVEL_CHANGED;

use crate::GameState;

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
                    .into(),
            )
            .add_startup_system(setup_fonts);
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
    mut egui_context: ResMut<EguiContext>,
    mut player: ResMut<PlayerState>,
    mut windows: ResMut<Windows>,
    mut pref: ResMut<Preferences>,
    mut audio_events: ResMut<AudioEvents>,
    mut action_queue: ResMut<ActionQueue>,
    mut game_recorder: ResMut<GameRecorder>,
    mut rec_string: Local<String>,
    mut player_last_dead: Local<bool>,
) {
    let mut player_died_this_frame = false;
    if !*player_last_dead && !player.alive() {
        player_died_this_frame = true;
        *player_last_dead = true;
    }

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
                        action_queue.push(Action::CheatCredits);
                    }
                    if ui.button("HEALTH").clicked() {
                        action_queue.push(Action::CheatHealth);
                    }
                    if ui.button("NEXT LEVEL").clicked() {
                        action_queue.push(Action::CheatLevel);
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
                        ("BLASTER", Turret::Blaster),
                        ("WAVE   ", Turret::Wave),
                        ("LASER  ", Turret::Laser),
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
                    let cost = player.blaster_upgrade_cost();
                    if ui.button(&format!("BLASTER {:8}", cost)).clicked() {
                        action_queue.push(Action::BlasterUpgrade);
                    }
                    let cost = player.wave_upgrade_cost();
                    if ui.button(&format!("WAVE    {:8}", cost)).clicked() {
                        action_queue.push(Action::WaveUpgrade);
                    }
                    let cost = player.laser_upgrade_cost();
                    if ui.button(&format!("LASERS  {:8}", cost)).clicked() {
                        action_queue.push(Action::LaserUpgrade);
                    }
                    ui.label("");

                    ui.label(&format!("GAME SPEED {:.2}", player.time_multiplier));
                    ui.horizontal(|ui| {
                        if ui.button(" -- ").clicked() {
                            action_queue.push(Action::GameSpeedDec);
                        }
                        if ui.button(" ++ ").clicked() {
                            action_queue.push(Action::GameSpeedInc);
                        }
                        if ui.button("PAUSE").clicked() {
                            action_queue.push(Action::GamePause);
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
                        **audio_events |= SFX_LEVEL_CHANGED;
                    }
                    if ui.button(" ++ ").clicked() {
                        pref.sfx = (pref.sfx + 0.1).min(3.0);
                        **audio_events |= SFX_LEVEL_CHANGED;
                    }
                    ui.label(&format!("SFX {:.1}", pref.sfx));
                });
                ui.horizontal(|ui| {
                    if ui.button(" -- ").clicked() {
                        pref.music = (pref.music - 0.1).max(0.0);
                        **audio_events |= MUSIC_LEVEL_CHANGED;
                    }
                    if ui.button(" ++ ").clicked() {
                        pref.music = (pref.music + 0.1).min(3.0);
                        **audio_events |= MUSIC_LEVEL_CHANGED;
                    }
                    ui.label(&format!("MUSIC {:.1}", pref.music));
                });
                ui.label("");
                if ui.button("RESTART GAME").clicked() {
                    action_queue.push(Action::RestartGame);
                    game_recorder.disable_rec = false;
                    game_recorder.play = false;
                    game_recorder.actions = ActionRecording::default();
                }
                if select_button(ui, "REPLAY", game_recorder.play) {
                    action_queue.push(Action::RestartGame);
                    game_recorder.play = true;
                    game_recorder.disable_rec = true;
                    game_recorder.play_head = 0;
                }
                if ui.text_edit_singleline(&mut *rec_string).changed() {
                    if let Ok(compressed) = base64::decode(rec_string.trim()) {
                        if let Ok(bytes) = decompress_size_prepended(&compressed) {
                            if let Ok(archived) =
                                rkyv::check_archived_root::<ActionRecording>(&bytes)
                            {
                                if let Ok(deserialized) =
                                    archived.deserialize(&mut rkyv::Infallible)
                                {
                                    game_recorder.actions = deserialized;
                                }
                            }
                        }
                    }
                }
                if player_died_this_frame || ui.button("GET REPLAY STRING").clicked() {
                    let bytes = rkyv::to_bytes::<_, 1024>(&game_recorder.actions).unwrap();
                    let compressed = compress_prepend_size(&bytes);
                    let base64_bytes = base64::encode(compressed);
                    *rec_string = base64_bytes;
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
