use bevy::diagnostic::DiagnosticsStore;
use bevy::window::PrimaryWindow;
use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::egui::Align2;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

/// Centralization plugin - Utilized to store all logic related to egui
pub struct SharedEgui;

impl Plugin for SharedEgui {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);

        app.add_systems(Update, (inspector_ui, shared_diagnostics_ui));

        // Bevy native diagnostics plugin - Those carry precious information
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_plugins(EntityCountDiagnosticsPlugin::default());
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

/// A shared diagnostics ui that diplays key perfomance informations, this works via acessing diagnostic store,
/// And grabing the given diagnostics from the plugin enums
fn shared_diagnostics_ui(
    mut contexts: bevy_egui::EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Some(egui_context) = contexts.try_ctx_mut() {
        egui::Window::new("Perfomance metrics")
            .default_open(false)
            .anchor(Align2::RIGHT_TOP, (-250.0, 0.0))
            .show(egui_context, |ui| {
                // We want to display the values only if we are able to grab them
                let fps = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                    .map(|fps| format!("FPS {:>4.0}", fps)) // Format the FPS if available
                    .unwrap_or_else(|| "FPS data not available".to_string()); // If not available return this string

                ui.label(fps);

                let entity_count = diagnostics
                    .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                    .and_then(|ec| ec.value())
                    .map(|fps| format!("ENTITY AMOUNT {:>4.0}", fps))
                    .unwrap_or_else(|| "Entity AMOUNT data not available".to_string());

                ui.label(entity_count);
            });
    }
}
