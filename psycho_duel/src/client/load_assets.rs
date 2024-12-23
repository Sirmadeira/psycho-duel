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
            "characters/visual_parts/suit_head.glb",
            "characters/visual_parts/soldier_head.glb",
            // Torsos
            "characters/visual_parts/scifi_torso.glb",
            "characters/visual_parts/soldier_torso.glb",
            // Legs
            "characters/visual_parts/witch_legs.glb",
            "characters/visual_parts/soldier_legs.glb",
            //Skeletons
            "characters/anim_skeletons/main_skeleton.glb"
        ),
        collection(typed, mapped)
    )]
    pub gltf_files: HashMap<String, Handle<Gltf>>,
}

/// Stores handles to all of the images utilized in our game
#[derive(AssetCollection, Resource, Reflect)]
#[reflect(Resource)]
pub struct ImagesCollection {
    #[asset(path = "images", collection(typed, mapped))]
    pub map: HashMap<String, Handle<Image>>,
}

impl Plugin for LoadAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Add in world debugger
        app.register_type::<GltfCollection>();
        app.register_type::<ImagesCollection>();
        app.add_loading_state(
            // Simple syntax sugar for our state transition and which specific collections we shall load, this can be used multiple times.
            LoadingState::new(ClientAppState::LoadingAssets)
                .load_collection::<GltfCollection>()
                .load_collection::<ImagesCollection>()
                .continue_to_state(ClientAppState::Game),
        );
    }
}
