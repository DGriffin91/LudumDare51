use bevy::math::*;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::{egui::FontDefinitions, *};

use crate::{player::PlayerState, turrets::Turret};

pub struct GameUI;
impl Plugin for GameUI {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .add_system(ui_sidebar)
            .add_startup_system(setup_fonts);
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
) {
    let window = windows.get_primary_mut().unwrap();
    egui::SidePanel::right("right_panel")
        .resizable(false)
        .min_width(window.width() * 0.17)
        .default_width(window.width() * 0.17)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(&format!("LEVEL {}", player.level as u32));
            let v = 1.0 - (player.level_time * 0.1 - player.level).fract();
            ui.label(&format!("NEXT LEVEL IN {:.2}", v * 10.0));
            ui.label("");
            ui.label(&format!("{} HEALTH", (player.health * 100.0) as u32));
            ui.label(&format!("{} CREDITS", player.credits));
            ui.label(&format!("{} KILLS", player.kills));
            ui.label("");
            ui.label("TURRETS");
            for (message, turret) in [
                ("BLASTER", Turret::Laser),
                ("WAVE", Turret::Shockwave),
                ("LASER", Turret::LaserContinuous),
            ] {
                if select_button(
                    ui,
                    &format!("{} {}", message, turret.cost()),
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
            if ui.button(&format!("BLASTER {}", cost)).clicked() && player.credits > cost {
                player.blaster_upgrade *= 1.1;
                player.credits -= cost;
            }
            let cost = (player.wave_upgrade.powi(2) * 10.0) as u64;
            if ui.button(&format!("WAVE {}", cost)).clicked() && player.credits > cost {
                player.wave_upgrade *= 1.1;
                player.credits -= cost;
            }
            let cost = (player.laser_upgrade.powi(2) * 10.0) as u64;
            if ui.button(&format!("LASERS {}", cost)).clicked() && player.credits > cost {
                player.laser_upgrade *= 1.1;
                player.credits -= cost;
            }
        });
}

pub fn setup_fonts(mut egui_context: ResMut<EguiContext>) {
    let mut fonts = FontDefinitions::default();

    for (_text_style, data) in fonts.font_data.iter_mut() {
        data.tweak.scale = 1.5;
    }
    egui_context.ctx_mut().set_fonts(fonts);
}
