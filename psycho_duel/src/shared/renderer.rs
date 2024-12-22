use crate::shared::client::{VisualInterpolateStatus, VisualInterpolationPlugin};
use bevy::prelude::*;
use lightyear::prelude::client::Predicted;

use super::protocol::PlayerMarker;

/// Simple plugin - Made to stores systems realted to visual interpolation between fixed updates
pub struct SharedRendererPlugin;

impl Plugin for SharedRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VisualInterpolationPlugin::<Transform>::default());
        app.observe(add_visual_interpolation_components::<Transform>);
    }
}

/// This plugins is reponsible for making the interpolationg among our visual components in fixed update
/// It needs to be shared because we also want to interpolate them in server
/// For better understanding go-to - https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html
fn add_visual_interpolation_components<T: Component>(
    trigger: Trigger<OnAdd, T>,
    query: Query<Entity, (With<T>, With<PlayerMarker>, With<Predicted>)>,
    mut commands: Commands,
) {
    if !query.contains(trigger.entity()) {
        return;
    }
    debug!("Adding visual interp component to {:?}", trigger.entity());
    commands
        .entity(trigger.entity())
        .insert(VisualInterpolateStatus::<T> {
            trigger_change_detection: true,
            ..default()
        });
}
