use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiSet};

use crate::client::ClientAppState;
/// Client focused egui
pub struct ClientEguiPlugin;

/// A resource that tracks whether egui wants focus on the current and previous frames,
///
/// The reason the previous frame's value is saved is because when you click inside an
/// egui window, Context::wants_pointer_input() still returns false once before returning
/// true. If the camera stops taking input only when it returns false, there's one frame
/// where both egui and the camera are using the input events, which is not desirable.
///
/// This is re-exported in case it's useful. I recommend only using input events if both
/// `prev` and `curr` are false.
#[derive(Resource, PartialEq, Eq, Default, Reflect, Debug)]
#[reflect(Resource)]
pub struct EguiWantsFocus {
    /// Whether egui wanted focus on the previous frame
    pub prev: bool,
    /// Whether egui wants focus on the current frame
    pub curr: bool,
}

impl Plugin for ClientEguiPlugin {
    fn build(&self, app: &mut App) {
        //Resouce registration
        app.init_resource::<EguiWantsFocus>();

        app.add_systems(Update, inspector_ui);
        app.add_systems(
            PostUpdate,
            check_if_egui_wants_focus.after(EguiSet::InitContexts),
        );

        // Debugging
        app.register_type::<EguiWantsFocus>();
    }
}

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
            ui.heading("Client debugging");
            egui::ScrollArea::both().show(ui, |ui| {
                ui.label("States inspector");
                bevy_inspector_egui::bevy_inspector::ui_for_state::<ClientAppState>(world, ui);
                ui.allocate_space(ui.available_size());
                // Wait for PR
                // bevy_inspector_egui::bevy_inspector::ui_for_state::<NetworkingState>(
                //     world, ui,
                // );
            })
        });
    } else {
        return;
    };
}

fn check_if_egui_wants_focus(
    mut contexts: EguiContexts,
    windows: Query<Entity, With<Window>>,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    // The window that the user is interacting with and the window that contains the egui context
    // that the user is interacting with are always going to be the same. Therefore, we can assume
    // that if any of the egui contexts want focus, then it must be the one that the user is
    // interacting with.
    let new_wants_focus = windows.iter().any(|window| {
        //Grabin context from most probably primary window
        if let Some(ctx) = contexts.try_ctx_for_entity_mut(window) {
            // Check if wants pointer input or keyboard input
            let mut value = ctx.wants_pointer_input() || ctx.wants_keyboard_input();
            value
        } else {
            false
        }
    });

    let new_res = EguiWantsFocus {
        prev: wants_focus.curr,
        curr: new_wants_focus,
    };
    info!("New res {:?}", new_res);
    wants_focus.set_if_neq(new_res);
}
