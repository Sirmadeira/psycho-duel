use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
/// Centralization plugin - Utilized to store all logic related to egui
pub struct SharedEgui;

impl Plugin for SharedEgui {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Update, inspector_ui);
    }
}

/// This is really close to what happens when you utilize worldinspectorplugin from bevy-inspector-egui
/// The difference is this guy is configurable meaning we can adjust him according to resolution and such
/// When querying for world, we are basically making this function independent of external systems.
/// We are going to run this guy doesnt matter what happens
fn inspector_ui(world: &mut World) {
    // This is basically comp_egui_context: Query<&EguiContext, With<PrimaryWindow>>,
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        info_once!("Manage to grab egui form world");
        // We clone here to avoid changin other eguis context
        let mut egui_context = egui_context.clone();

        //  Imagine this as nesting so first comes window, when we do,
        // add_content closure ui we are ensuring that scroll area is child of window.
        // All you need to do is add more and more .show to make heavier nests. And call ui a lot if you want to make buttons and such
        egui::Window::new("WorldInspector")
            // We gonna spawn it closed
            .default_open(false)
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // This is equivalent to "world inspector plugin"
                    bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

                    egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                        bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                            world, ui,
                        );
                    });
                    // egui::CollapsingHeader::new("States").show(ui, |ui| {
                    //     // Wair for PR
                    //     // bevy_inspector_egui::bevy_inspector::ui_for_state::<NetworkingState>(
                    //     //     world, ui,
                    //     // );
                    // });
                })
            });
    } else {
        return;
    };
}
