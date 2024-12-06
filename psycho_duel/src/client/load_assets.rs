use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;

use super::ClientAppState;

/// Essential plugin responsible for loading assets in our games. It also performs the first state transition
/// from LoadingAssets to ClientAppState in the ClientAppState state.
pub struct LoadAssetsPlugin;

/// Gltf collection, currently stores  all assets that are imported gltfs.
#[derive(AssetCollection, Resource, Reflect)]
#[reflect(Resource)]
pub struct GltfCollection {
    #[asset(
        paths(
            // Weapons
            "weapons/katana.glb",
            // Full characters - This guy is off for now to avoid annoying warning
            // "characters/character_mesh.glb",
            // Heads
            "characters/parts/suit_head.glb",
            "characters/parts/soldier_head.glb",
            // Torsos
            "characters/parts/scifi_torso.glb",
            "characters/parts/soldier_torso.glb",
            // Legs
            "characters/parts/witch_legs.glb",
            "characters/parts/soldier_legs.glb",
            //Skeletons
            "characters/parts/main_skeleton.glb"
        ),
        collection(typed, mapped)
    )]
    pub gltf_files: HashMap<String, Handle<Gltf>>,
}

/// Stores handles to all of the images utilized in our game
#[derive(AssetCollection, Resource, Reflect)]
#[reflect(Resource)]
pub struct Images {
    #[asset(path = "images", collection(typed, mapped))]
    pub map: HashMap<String, Handle<Image>>,
}

impl Plugin for LoadAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Add in world debugger
        app.register_type::<GltfCollection>();
        app.register_type::<Images>();
        app.add_loading_state(
            // Simple syntax sugar for our state transition and which specific collections we shall load, this can be used multiple times.
            LoadingState::new(ClientAppState::LoadingAssets)
                .continue_to_state(ClientAppState::Game)
                .load_collection::<GltfCollection>()
                .load_collection::<Images>(),
        );
    }
}
