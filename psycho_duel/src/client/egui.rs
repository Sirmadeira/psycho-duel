use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};

use crate::client::ClientAppState;
/// Client focused egui
pub struct ClientEguiPlugin;

fn inspector_ui(world: &mut World) {
    // This is basically comp_egui_context: Query<&EguiContext, With<PrimaryWindow>>,
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        info_once!("Forming states egui");
        // We clone here to avoid changin other eguis context
        let mut egui_context = egui_context.clone();

        //  Imagine this as nesting so first comes window, when we do,
        // add_content closure ui we are ensuring that scroll area is child of window.
        // All you need to do is add more and more .show to make heavier nests. And call ui a lot if you want to make buttons and such
        egui::SidePanel::right("right_panel").show(egui_context.get_mut(), |ui| {
            ui.heading("Client focused debugging");
            ui.label("States inspector");
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_state::<ClientAppState>(world, ui);
            })
            // Wait for PR
            // bevy_inspector_egui::bevy_inspector::ui_for_state::<NetworkingState>(
            //     world, ui,
            // );
        });
    } else {
        return;
    };
}

impl Plugin for ClientEguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, inspector_ui);
    }
}
