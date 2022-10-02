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

const SELECTED_COLOR: Color32 = Color32::from_rgb(96, 96, 96);
const DESELECTED_COLOR: Color32 = Color32::from_rgb(64, 64, 64);

fn turret_button(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
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
            ui.label(&format!("{} CREDITS", player.credits));
            ui.label(&format!("{} KILLS", player.kills));
            for (message, turret) in [
                ("LASER TURRET", Turret::Laser),
                ("WAVE TURRET", Turret::Shockwave),
            ] {
                if turret_button(ui, message, player.turret_to_place == Some(turret)) {
                    player.turret_to_place = Some(turret);
                }
            }
        });
}

pub fn setup_fonts(mut egui_context: ResMut<EguiContext>) {
    let mut fonts = FontDefinitions::default();

    for (_text_style, data) in fonts.font_data.iter_mut() {
        data.tweak.scale = 2.0;
        //*size = 25.0;
    }
    egui_context.ctx_mut().set_fonts(fonts);
}
