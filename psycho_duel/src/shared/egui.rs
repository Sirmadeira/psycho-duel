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
    // This is basically comp_egui_context: Query<&EguiContext, With<PrimaryWindow>>, here we are grabbin the egui context from our primary window
    // Logically this is supposed to be the only context available unless you are doing some pretty trippy things,
    //We do it like this so we can grab world for bevy inspector
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        info_once!("Manage to grab egui from world");

        // We clone here to ensure the context remains concurrrently queryable I think ?
        let mut egui_context = egui_context.clone();

        // Imagine this as nesting so first comes window, so when we do add_content closure ui we are ensuring that scroll area is child of window.
        // All you need to do is add more and more .show to make heavier nests. And call ui a lot if you want to make buttons and such
        // Egui context.get_mut grab the underlying context it is a handy way of grab self without the annoyance of self arguments mid usage
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(250.0)
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.heading("World inspector");
                    // This is equivalent to "world inspector plugin"
                    bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
                    egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                        bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(
                            world, ui,
                        );
                    });
                    // Makes him unlimited
                    ui.allocate_space(ui.available_size());
                })
            });
    } else {
        return;
    };
}
