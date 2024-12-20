use std::ops::DerefMut;

use crate::client::{ClientAppState, CoreEasyClient};
use crate::shared::protocol::Currency;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};
use client::Predicted;
use lightyear::prelude::*;
use lightyear::shared::replication::components::Controlled;

use super::load_assets::GltfCollection;
use super::protocol::*;
use super::CommonChannel;
/// Client focused egui
pub struct ClientEguiPlugin;

/// Gives me the current parts in ui we are able to customize,utilized in combo box button
/// Locals - Variables that are unique to a system, and only have a reference to that system.
#[derive(PartialEq, Debug, Default, Serialize, Deserialize, Clone)]
pub enum Parts {
    #[default]
    Head,
    Torso,
    Leg,
}

/// Static string references =  dont expect this variable to change mid system running
/// Technically this are the paths to all available items, just increase this guy to adjust ui button
const HEAD_PATHS: [&'static str; 2] = [
    "characters/visual_parts/suit_head.glb",
    "characters/visual_parts/soldier_head.glb",
];

const TORSO_PATHS: [&'static str; 2] = [
    "characters/visual_parts/scifi_torso.glb",
    "characters/visual_parts/soldier_torso.glb",
];

const LEG_PATHS: [&'static str; 2] = [
    "characters/visual_parts/witch_legs.glb",
    "characters/visual_parts/soldier_legs.glb",
];

/// Carrier of information usefull for our char customizer
#[derive(Event, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChangeCharEvent {
    /// Client id of who asked for adjustment1
    pub client_id: ClientId,
    /// Body part specific part that we hope to adjust
    pub body_part: Parts,
    /// The item that needs to change
    pub item: Item,
}

impl Plugin for ClientEguiPlugin {
    fn build(&self, app: &mut App) {
        //Events
        app.add_event::<ChangeCharEvent>();

        //In update by default
        app.add_systems(
            Update,
            (inspector_ui, char_customizer_ui, currency_ui, store_ui),
        );
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
            ui.heading("Client debugger");
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("States inspector");
                ui.add_space(10.0);
                // Creates a little division strand
                ui.separator();
                // Creates an empty space
                ui.add_space(10.0);
                bevy_inspector_egui::bevy_inspector::ui_for_state::<ClientAppState>(world, ui);
                // Wait for PR
                // bevy_inspector_egui::bevy_inspector::ui_for_state::<NetworkingState>(
                //     world, ui,
                // );

                ui.heading("Additional Resources");
                ui.add_space(10.0);
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

/// A developer egui utilized to limit test our game character customizer
fn char_customizer_ui(
    mut contexts: bevy_egui::EguiContexts,
    local_player: Query<&PlayerId, (With<Predicted>, With<Controlled>)>,
    mut selected_button: Local<Parts>,
    mut commands: Commands,
) {
    // Only should appear if replication ocurred
    if let Ok(player_id) = local_player.get_single() {
        // Egui context
        if let Some(egui_context) = contexts.try_ctx_mut() {
            // Cleaner dereferencing
            let selected_button = selected_button.deref_mut();
            egui::Window::new("Char customizer")
                .default_open(false)
                .show(egui_context, |ui| {
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

                        // Here we intake ref because we dont wannt to consume file path const
                        let items: Vec<Item> = paths
                            .iter()
                            .map(|&path| Item::new_from_filepath(path))
                            .collect();

                        // For each item, we make a button  capable of sending an event with it is given file_path
                        for item in items.iter() {
                            let item_name = item.name.to_string();

                            if ui.button(&item_name).clicked() {
                                let client_id = player_id.id;
                                // Selected button is technically also the definer of what body part we want to change didnt name it differently because of egui context
                                let body_part = &selected_button;
                                send_trigger_event(&mut commands, body_part, item, client_id);
                            }
                        }
                    })
                });
        }
    }
}

/// Callable function utilized to avoid repetitition in char custumizedr ui
fn send_trigger_event(
    commands: &mut Commands,
    body_part: &Parts,
    item: &Item,
    client_id: ClientId,
) {
    //We dont actually want event here we just wanna trigger observer
    commands.trigger(ChangeCharEvent {
        client_id: client_id,
        body_part: body_part.clone(),
        item: item.clone(),
    });
    info!(
        "Change char event sent successfully! Part to change {} client {}",
        item, client_id
    );
}

/// Egui responsible to test features gaining currency, losing currency
fn currency_ui(
    mut contexts: bevy_egui::EguiContexts,
    mut player_q: Query<(&PlayerId, &mut Currency), (With<Predicted>, With<Controlled>)>,
    mut connection_manager: ResMut<ClientConnectionManager>,
) {
    // Only should appear if replication already ocurred
    // It is okay we can mutate locally, nonetheless server will override it via replication if not okaied validation
    if let Ok((player_id, mut current_currency)) = player_q.get_single_mut() {
        // Grab primary window ctx
        if let Some(egui_context) = contexts.try_ctx_mut() {
            // Use the egui context
            egui::Window::new("Currency mechanics")
                .default_open(false)
                .show(egui_context, |ui| {
                    ui.heading(format!("Total amount {}", current_currency.amount));
                    if ui.button("Gain currency").clicked() {
                        current_currency.add(10.0);

                        // Send event here
                        let _ = connection_manager.send_message::<CommonChannel, SaveMessage>(
                            &mut SaveMessage {
                                id: player_id.id,
                                change_char: None,
                                change_currency: Some(current_currency.clone()),
                                change_inventory: None,
                            },
                        );
                    }
                    if ui.button("Lose currency").clicked() {
                        // Adjust currency logic and send event here
                        current_currency.sub(10.0);

                        let _ = connection_manager.send_message::<CommonChannel, SaveMessage>(
                            &mut SaveMessage {
                                id: player_id.id,
                                change_char: None,
                                change_currency: Some(current_currency.clone()),
                                change_inventory: None,
                            },
                        );
                    }
                });
        }
    }
}

/// Egui representing our store mechanics things like buying items selling them should occur here
fn store_ui(
    mut contexts: bevy_egui::EguiContexts,
    gltf_collection: Option<Res<GltfCollection>>,
    mut player_q: Query<
        (&PlayerId, &mut Currency, &mut Inventory),
        (With<Predicted>, With<Controlled>),
    >,
    mut connection_manager: ResMut<ClientConnectionManager>,
) {
    // Only show the store if assets are available and the player is replicated
    if let Some(gltf_collection) = gltf_collection {
        if let Ok((player_id, mut player_money, mut player_inv)) = player_q.get_single_mut() {
            // Egui context

            if let Some(egui_context) = contexts.try_ctx_mut() {
                // Get the available items from the GLTF collection
                let items: Vec<Item> = gltf_collection
                    .gltf_files
                    .keys()
                    .into_iter()
                    .map(|file_path| Item::new_from_filepath(file_path))
                    .collect();

                // Render the store UI
                egui::Window::new("Store")
                    .default_open(false)
                    .show(egui_context, |ui| {
                        egui::ScrollArea::both().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Buy section
                                render_buy_section(
                                    ui,
                                    &items,
                                    &player_id,
                                    &mut player_inv,
                                    &mut player_money,
                                    &mut connection_manager,
                                );

                                // Sell section
                                render_sell_section(
                                    ui,
                                    &player_id,
                                    &mut player_inv,
                                    &mut player_money,
                                    &mut connection_manager,
                                );
                            });
                        });
                    });
            }
        }
    }
}

/// Render the "Buy" section
fn render_buy_section(
    ui: &mut egui::Ui,
    items: &[Item],
    player_id: &PlayerId,
    player_inv: &mut Inventory,
    player_money: &mut Currency,
    connection_manager: &mut ResMut<ClientConnectionManager>,
) {
    ui.vertical(|ui| {
        ui.heading("Buy items");
        for item in items {
            ui.horizontal(|ui| {
                // Button to buy the item
                let item_name = item.name.to_string();
                if ui.button(&item_name).clicked() {
                    player_inv.insert_item(item.clone());
                    player_money.sub(item.item_type.value());
                    info!("Bought action, for item {} price {}", item_name, {
                        item.item_type.value()
                    });
                    let _ = connection_manager.send_message::<CommonChannel, SaveMessage>(
                        &mut SaveMessage {
                            id: player_id.id,
                            change_char: None,
                            change_currency: Some(player_money.clone()),
                            change_inventory: Some(player_inv.clone()),
                        },
                    );
                }

                // Item price
                ui.label(format!("Cost: {}", item.item_type.value()));
            });
        }
    });
}

/// Render the "Sell" section
fn render_sell_section(
    ui: &mut egui::Ui,
    player_id: &PlayerId,
    player_inv: &mut Inventory,
    player_money: &mut Currency,
    connection_manager: &mut ResMut<ClientConnectionManager>,
) {
    ui.vertical(|ui| {
        ui.heading("Sell your items");

        // Collect inventory items into a temporary vector to avoid borrowing issues
        let items: Vec<_> = player_inv.items.values().cloned().collect();
        for item in items {
            ui.horizontal(|ui| {
                // Button to sell the item
                let item_name = item.name.to_string();
                if ui.button(&item_name).clicked() {
                    player_inv.remove_item(&item);
                    player_money.add(item.item_type.value());
                    info!("Sell action, for item {} price {}", item_name, {
                        item.item_type.value()
                    });

                    let _ = connection_manager.send_message::<CommonChannel, SaveMessage>(
                        &mut SaveMessage {
                            id: player_id.id,
                            change_char: None,
                            change_currency: Some(player_money.clone()),
                            change_inventory: Some(player_inv.clone()),
                        },
                    );
                }

                // Item price
                ui.label(format!("Cost: {}", item.item_type.value()));
            });
        }
    });
}
