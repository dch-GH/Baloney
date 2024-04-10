use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::{
        camera::PhysicalCameraParameters,
        extract_component::ExtractComponent,
        mesh::morph::MeshMorphWeights,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::*,
    },
};
use bevy_sprite3d::Sprite3dParams;

use crate::{enemy::EnemyKind, tilemap::TILE_SIZE};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraParameters(pub PhysicalCameraParameters);

#[derive(Resource, Default)]
pub struct GameResourceHandles {
    pub mossy_cobble: Handle<StandardMaterial>,
    pub cobble_material: Handle<StandardMaterial>,
    pub brick_material: Handle<StandardMaterial>,
    pub dice_material: Handle<StandardMaterial>,
    pub cube: Handle<Mesh>,
    pub plane: Handle<Mesh>,
    pub dice_mesh: Handle<Mesh>,
    pub render_texture: Handle<Image>,
    pub font: Handle<Font>,
    pub enemy_sprites: HashMap<EnemyKind, Handle<Image>>,
}

pub fn init_resources(
    mut cmd: Commands,
    mut assets: ResMut<AssetServer>,
    mut resources: ResMut<GameResourceHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let cobble_texture_handle: Handle<Image> = assets.load("cobble.png");
    let mossy_cobble_texture_handle: Handle<Image> = assets.load("mossy_cobble.png");
    let brick_texture_handle: Handle<Image> = assets.load("brick.png");
    let dice_texture_handle: Handle<Image> = assets.load("cardsMedium_tilemap_packed.png");

    let cobble_material = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(cobble_texture_handle.clone()),

        unlit: false,
        ..default()
    });

    let mossy_cobble_material = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(mossy_cobble_texture_handle.clone()),

        unlit: false,
        ..default()
    });

    let brick_material = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(brick_texture_handle.clone()),
        perceptual_roughness: 0.85,
        metallic: 0.01,
        unlit: false,
        ..default()
    });

    let dice_material = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(dice_texture_handle.clone()),
        unlit: false,
        ..default()
    });

    let cube_handle = meshes.add(Cuboid {
        half_size: Vec3::splat(TILE_SIZE / 2.0),
    });

    let plane_handle = meshes.add(Plane3d::default().mesh().size(TILE_SIZE, TILE_SIZE));
    let dice_mesh: Handle<Mesh> = assets.load("meshes/dice.obj");

    let size = Extent3d {
        width: 160 * 2,
        height: 120 * 2,
        ..default()
    };

    let mut render_texture = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    render_texture.resize(size);
    let render_texture_handle = images.add(render_texture);

    resources.cobble_material = cobble_material;
    resources.brick_material = brick_material;
    resources.mossy_cobble = mossy_cobble_material;
    resources.dice_material = dice_material;
    resources.dice_mesh = dice_mesh;
    resources.cube = cube_handle;
    resources.plane = plane_handle;
    resources.render_texture = render_texture_handle;
    resources.font = assets.load("fonts/Minecraftchmc-dBlX.ttf");

    let mut dict: HashMap<EnemyKind, Handle<Image>> = HashMap::new();
    dict.insert(EnemyKind::Skull, assets.load("enemy_sprites/skull.png"));

    resources.enemy_sprites = dict;
}

#[derive(Resource, Default)]
pub struct UserSettings {
    pub mouse_sens: f32,
}
