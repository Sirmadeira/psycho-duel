use bevy::{
    prelude::*,
    utils::{Duration, HashMap},
};

use super::{load_assets::GltfCollection, ClientAppState};

/// Plugin responsible for animation on client
pub struct ClientAnimationPlugin;

/// Reponsible for giving me all the current available animations
#[derive(Resource, Default, Reflect)]
pub struct Animations {
    pub graph_handle: Handle<AnimationGraph>,
    pub named_node: HashMap<String, AnimationNodeIndex>,
}

impl Plugin for ClientAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Animations>();

        app.add_systems(OnEnter(ClientAppState::Game), setup_animation_resource);

        app.add_systems(Update, play_animations);
    }
}

/// Skeleton entities are the ones repsonsible for carrying all of our animations
/// Here we are going to create a resource that easily gives me the available animations
fn setup_animation_resource(
    gltf_collection: Res<GltfCollection>,
    mut animation: ResMut<Animations>,
    mut graphs_assets: ResMut<Assets<AnimationGraph>>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    let skeleton_han = gltf_collection
        .gltf_files
        .get("characters/anim_skeletons/main_skeleton.glb")
        .expect("To have animations in main skeleton");

    let skeleton = gltf_assets.get(skeleton_han).unwrap();

    let mut animation_graph = AnimationGraph::new();

    for (animation_name, clip) in skeleton.named_animations.iter() {
        // Making nodes
        let node = animation_graph.add_clip(clip.clone(), 1.0, animation_graph.root);
        // Making a simple hashmap acessor to those nodes
        animation
            .named_node
            .insert(animation_name.to_string(), node);
    }

    let han_graph = graphs_assets.add(animation_graph);

    animation.graph_handle = han_graph;
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn play_animations(
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    animations: Res<Animations>,
    mut commands: Commands,
) {
    for (entity, mut player) in players.iter_mut() {
        info!("Add animation");
        let mut transitions = AnimationTransitions::new();
        let new_animation = animations.named_node.get("Sword_Slash").unwrap();
        transitions
            .play(&mut player, *new_animation, Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(transitions)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()));
    }
}
