use super::ClientAppState;
use super::{camera::MarkerPrimaryCamera, load_assets::ImagesCollection};
use bevy::core_pipeline::Skybox;
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};

/// Plugin responsible for carrying our skybox. Perhaps later we can think about making it into a shader
/// As this can become comple really fast, instead of utilizing the camera plugin we gonna separa the logic for reasons
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientAppState::Game), reinterpret_skysheet_image);
    }
}

/// The workflow for this goes as follow\
/// -> Make an image that represents your atmosphere a good approach is just make a blender sphere and add texture to it
/// -> Then save that texture as PNG and convert it into a cubemap via https://jaxry.github.io/panorama-to-cubemap/
/// -> After that create a spritesheet from it - https://www.codeandweb.com/free-sprite-sheet-packer
/// -> And finally just add it in images and test accordingly
fn reinterpret_skysheet_image(
    mut images: ResMut<Assets<Image>>,
    loaded_images: Res<ImagesCollection>,
    cam_q: Query<Entity, With<MarkerPrimaryCamera>>,
    mut commands: Commands,
) {
    if let Ok(cam_ent) = cam_q.get_single() {
        // Sprite sheet is just white, skysheets gives a cool cube effect
        let image_handle = loaded_images
            .map
            .get("images/skysheet.png")
            .expect("To find images/skysheet.png loaded");

        let sky_image = images.get_mut(image_handle).unwrap();

        // This convers the skysheet into a skybox image
        if sky_image.texture_descriptor.array_layer_count() == 1 {
            sky_image.reinterpret_stacked_2d_as_array(sky_image.height() / sky_image.width());
            sky_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
            sky_image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..Default::default()
            });
        }

        let skybox = Skybox {
            image: image_handle.clone_weak(),
            brightness: 500.0,
            ..default()
        };

        commands.entity(cam_ent).insert(skybox);
    }
}
