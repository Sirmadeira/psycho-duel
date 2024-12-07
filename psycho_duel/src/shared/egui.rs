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
/// The difference is this guy is configurable meaning we can adjust him according to resolution changes and such
/// Egui contexts is a system_parameter extremely usefull for reducing boilerplate
fn inspector_ui(world: &mut World) {
    // This is basically comp_egui_context: Query<&EguiContext, With<PrimaryWindow>>,
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        info_once!("Manage to grab egui form world");
        // We clone here to avoid changin egui context directly
        let mut egui_context = egui_context.clone();

        // Imagine this as writing your egui screen so first comes side panel, when we do,
        // add_content closure ui we are ensuring that scroll area is child of that side panel.
        // All you need to do is add more and more .show to make heavier nests.
        // If you want to add more mechanics just call egui::some_enum or ui::some_enum. Some enum being the thousands of options available
        // Important usually people use window - but since I dont our camera systems or input systems to be considered well panel it is.
        egui::SidePanel::left("World debug panel")
            .resizable(true)
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    // This is equivalent to "world inspector plugin"
                    bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

                    egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                        ui.set_width(400.0);
                        bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                            world, ui,
                        );
                    });
                })
            });
    } else {
        return;
    };
}
