use bevy::prelude::*;
use lightyear::prelude::client::Predicted;

use super::{
    load_assets::GltfCollection,
    protocol::{PlayerId, PlayerVisuals},
};

pub struct ClientPlayerPlugin;

/// Whenever we spawn an entity with player visuals, we are going to check if she is predicted if so.
/// We are going to spawn their given scenes.
/// IMPORTANT - Observers are essential here, because them we dont need to worry, about resource management.
fn render_predicted_player(
    trigger: Trigger<OnAdd, PlayerVisuals>,
    player_comp: Query<(&PlayerId, &PlayerVisuals, Has<Predicted>)>,
    gltf_collection: Res<GltfCollection>,
    gltfs: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    let parent = trigger.entity();
    // Avoids error - https://bevyengine.org/learn/errors/b0004/
    commands.entity(parent).insert(SpatialBundle::default());

    if let Ok((player_id, player_visuals, is_predicted)) = player_comp.get(parent) {
        if is_predicted {
            for file_path in player_visuals.iter_visuals() {
                if let Some(entity) =
                    spawn_visual_scene(file_path, &gltf_collection, &gltfs, &mut commands)
                {
                    info!(
                        "Spawning visuals for client_id {},{}, entity {} and making them children fo",
                        player_id.id, file_path, entity
                    );
                    commands.entity(entity).set_parent(parent);
                }
            }
        }
    }
}

/// Nested function necessary that intakes a given a file_path string and spawns the given scene for it.
fn spawn_visual_scene(
    file_path: &String,
    gltf_collection: &Res<GltfCollection>,
    gltfs: &Res<Assets<Gltf>>,
    commands: &mut Commands,
) -> Option<Entity> {
    if let Some(gltf) = gltf_collection.gltf_files.get(file_path) {
        if let Some(loaded_gltf) = gltfs.get(gltf) {
            let scene = loaded_gltf.scenes[0].clone_weak();
            let id = commands
                .spawn(SceneBundle {
                    scene: scene,
                    ..default()
                })
                .id();
            Some(id)
        } else {
            warn!("Didnt manage to find gltf handle in our gltf asset");
            None
        }
    } else {
        warn!(
            "Couldnt find the given file path {} in our gltf collection did you forget to add him?",
            file_path
        );
        None
    }
}

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        // Observes because This is not gonna run all the time just when we connect and replicate the entities
        app.observe(render_predicted_player);
        //Update because it is waiting for added predicted to appear
    }
}
