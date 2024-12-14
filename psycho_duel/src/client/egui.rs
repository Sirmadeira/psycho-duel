use std::ops::DerefMut;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiSet};
use lightyear::prelude::*;

use crate::client::{ClientAppState, CoreEasyClient};
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

/// Gives me the current parts in ui we are able to customize,utilized in combo box button
/// Locals - Variables that are unique to a system, and only have a reference to that system.
#[derive(PartialEq, Debug, Default, Clone)]
pub enum Parts {
    #[default]
    Head,
    Torso,
    Leg,
}

/// Statig string references I dont expect this variable to change mid system running as I dont see any reason why
const HEAD_PATHS: [&'static str; 2] = [
    "characters/parts/suit_head.glb",
    "characters/parts/soldier_head.glb",
];

const TORSO_PATHS: [&'static str; 2] = [
    "characters/parts/scifi_torso.glb",
    "characters/parts/soldier_torso.glb",
];

const LEG_PATHS: [&'static str; 2] = [
    "characters/parts/witch_legs.glb",
    "characters/parts/soldier_legs.glb",
];

/// Carrier of information usefull for our char customizer
#[derive(Event, Debug)]
pub struct ChangeCharEvent {
    /// Client id of who asked for adjustment1
    pub client_id: ClientId,
    /// Body part specific part that we hope to adjust
    pub body_part: Parts,
    /// A file path for new part
    pub path_to_part: String,
}

impl Plugin for ClientEguiPlugin {
    fn build(&self, app: &mut App) {
        //Resouce registration
        app.init_resource::<EguiWantsFocus>();
        //Events
        app.add_event::<ChangeCharEvent>();

        //In update by default
        app.add_systems(Update, (inspector_ui, char_customizer_ui));

        // In post update to ensure that in preupdate we have the correct prev and curr value
        // After init contexts because we expect the Egui to exist.
        app.add_systems(
            PostUpdate,
            check_if_egui_wants_focus.after(EguiSet::InitContexts),
        );

        // Debugging
        app.register_type::<EguiWantsFocus>();
    }
}

fn inspector_ui(world: &mut World) {
    // This is basically comp_egui_context: Query<&EguiContext, With<PrimaryWindow>>, here we are grabbin the egui context from our primary window
    // Logically this is supposed to be the only context available unless you are doing some pretty trippy things,
    // We do it like this so we can grab world for bevy inspector
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        info_once!("Forming states egui");
        let mut egui_context = egui_context.clone();

        // Imagine this as nesting so first comes window, so when we do add_content closure ui we are ensuring that scroll area is child of window.
        // All you need to do is add more and more .show to make heavier nests. And call ui a lot if you want to make buttons and such
        // Egui context.get_mut grab the underlying context it is a handy way of grab self without the annoyance of self arguments mid usage
        egui::SidePanel::right("right_panel").show(egui_context.get_mut(), |ui| {
            ui.heading("Client debugging");
            egui::ScrollArea::both().show(ui, |ui| {
                ui.label("States inspector");
                // Creates an empty space
                ui.add_space(10.0);
                bevy_inspector_egui::bevy_inspector::ui_for_state::<ClientAppState>(world, ui);
                // Wait for PR
                // bevy_inspector_egui::bevy_inspector::ui_for_state::<NetworkingState>(
                //     world, ui,
                // );

                ui.heading("Additional Resources");
                ui.add_space(10.0);
                // Creates a little division strand
                ui.separator();
                // Here we are using this little ui_push id to avoid same widget id for both of them FIX - Little red warning hehe
                ui.push_id("core_easy_client", |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("Your Client").size(14.0),
                    ));
                    bevy_inspector_egui::bevy_inspector::ui_for_resource::<CoreEasyClient>(
                        world, ui,
                    );
                });

                // Makes dragable panel size unlimited
                ui.allocate_space(ui.available_size());
            });
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
    // The window that the user is interacting with and the window that contains the egui context are almost always the same. Therefore, we can assume
    // that if any of the egui contexts want focus, then it must be the one that the user is interacting with.
    let new_wants_focus = windows.iter().any(|window| {
        if let Some(ctx) = contexts.try_ctx_for_entity_mut(window) {
            // Check if wants pointer input or keyboard input
            let value = ctx.wants_pointer_input() || ctx.wants_keyboard_input();
            value
        } else {
            false
        }
    });

    let new_res = EguiWantsFocus {
        prev: wants_focus.curr,
        curr: new_wants_focus,
    };
    // info!("New res {:?}", new_res);
    wants_focus.set_if_neq(new_res);
}

/// A developer egui utilized to limit test our game character customizer
fn char_customizer_ui(world: &mut World, mut selected_button: Local<Parts>) {
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    {
        // Clone to grab it concurrently
        let mut egui_context = egui_context.clone();

        // Cleaner dereferencing
        let selected_button = selected_button.deref_mut();

        egui::Window::new("Char customizer").show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.label("Part to change");
                // For some unknow reason combobox requires hash id, which I just didnt feel like writing so from empty label it is
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", selected_button))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(selected_button, Parts::Head, "Head");
                        ui.selectable_value(selected_button, Parts::Torso, "Torso");
                        ui.selectable_value(selected_button, Parts::Leg, "Leg");
                    });

                ui.label("Available parts");
                // Matching pattern to grab pre defined const paths
                let paths = match selected_button {
                    Parts::Head => &HEAD_PATHS,
                    Parts::Torso => &TORSO_PATHS,
                    Parts::Leg => &LEG_PATHS,
                };

                // For each part in path, we make a button  capable of sending an event with it is given file_path
                for path_to_part in paths.iter() {
                    // Slicing for pretty :)
                    let sliced_str = path_to_part
                        .split("/")
                        .last()
                        .unwrap_or(&path_to_part)
                        .to_string();

                    if ui.button(sliced_str).clicked() {
                        // Grab resource easy client for client_id
                        if let Some(client_holder) = world.get_resource::<CoreEasyClient>() {
                            let client_id = client_holder.client_id;
                            // Selected button is technically also the definer of what body part we want to change didnt name it differently because of egui context
                            let body_part = selected_button.clone();
                            send_change_event(world, body_part,path_to_part, client_id);
                        } else {
                            warn!("We dont expect to be able to customize character offline ! As we still need to make our save mechanic")
                        }
                    }
                }
            })
        });
    }
}

/// Nested function utilized to avoid repetitition
fn send_change_event(world: &mut World, body_part: Parts, path_to_part: &str, client_id: ClientId) {
    //We dont actually want event here we just wanna trigger observer
    if let Some(_) = world.get_resource_mut::<Events<ChangeCharEvent>>() {
        world.trigger(ChangeCharEvent {
            client_id: client_id,
            body_part: body_part,
            path_to_part: path_to_part.to_string(),
        });
        info!(
            "Change char event sent successfully! Part to change {} client {}",
            path_to_part, client_id
        );
    } else {
        warn!("ChangeCharEvent is not registered. Did you forget to add it with `.add_event()`?");
    }
}
