use bevy::{
    prelude::*,
    utils::{Duration, HashMap},
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::client::Predicted;

use super::{load_assets::GltfCollection, protocol::PlayerActions, ClientAppState};

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

        // Should only occur when entering game state
        app.add_systems(OnEnter(ClientAppState::Game), setup_animation_resource);

        // Should check this forever
        app.add_systems(Update, add_animation_components);

        app.add_systems(Update, movement_animations);
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
        .get("characters/anim_skeletons/def_m_main_skeleton.glb")
        .expect("To find animation skeleton");

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

/// Adds animation component essential to play animations TO EVERY ENTITY THAT HAS AN Animation player
/// Currently add animation transitions, and an animation graph handle. Necessary because BEVY
/// Currently - Only predicted players should have it
fn add_animation_components(
    animation_player: Query<Entity, Added<AnimationPlayer>>,
    animations: Res<Animations>,
    mut commands: Commands,
) {
    for animated in animation_player.iter() {
        commands
            .entity(animated)
            .insert(AnimationTransitions::new())
            .insert(AnimationGraphHandle(animations.graph_handle.clone()));
    }
}

/// Queries predicted entities with the component player action as this occcurs plays the animation according to the received type
fn movement_animations(
    mut action_state: Query<
        (
            &ActionState<PlayerActions>,
            &mut AnimationTransitions,
            &mut AnimationPlayer,
        ),
        With<Predicted>,
    >,
    animations: Res<Animations>,
) {
    for (action, mut animation_transitions, mut animation_player) in action_state.iter_mut() {
        let new_animation = if action.just_pressed(&PlayerActions::Forward) {
            animations.named_node.get("KNEELESS_FRONT_WALK")
        } else if action.just_pressed(&PlayerActions::Backward) {
            animations.named_node.get("KNEELESS_BACK_WALK")
        } else if action.just_pressed(&PlayerActions::Left) {
            animations.named_node.get("KNEELESS_LEFT_WALK")
        } else if action.just_pressed(&PlayerActions::Right) {
            animations.named_node.get("KNEELESS_RIGHT_WALK")
        } else {
            None
        };
        if let Some(new_animation) = new_animation {
            animation_transitions
                .play(&mut animation_player, *new_animation, Duration::ZERO)
                .repeat();
        }
    }
}
