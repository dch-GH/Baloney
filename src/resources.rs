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
use bevy_rapier3d::parry::partitioning;
use bevy_sprite3d::Sprite3dParams;

use crate::{enemy::EnemyKind, tilemap::TILE_SIZE, utils::ez_str};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraParameters(pub PhysicalCameraParameters);

pub(crate) struct TileMaterial;
pub(crate) struct GameObjectMaterial;

#[derive(Eq, PartialEq, Hash)]
pub enum MaterialName {
    Cobble,
    MossyCobble,
    Brick,
    RoughStone,
    Dice,
}

#[derive(Resource, Default)]
pub struct GameResourceHandles {
    pub materials: HashMap<MaterialName, Handle<StandardMaterial>>,
    pub cube: Handle<Mesh>,
    pub plane: Handle<Mesh>,
    pub dice_mesh: Handle<Mesh>,
    pub render_texture: Handle<Image>,
    pub font: Handle<Font>,
    pub enemy_sprites: HashMap<EnemyKind, Handle<Image>>,
}

impl GameResourceHandles {
    /// Shortcut fn that returns a clone of a material.
    /// Uses .unwrap() so beware :).
    pub(crate) fn get_material(&self, name: MaterialName) -> Handle<StandardMaterial> {
        self.materials.get(&name).unwrap().clone()
    }
}

pub(crate) fn init(mut app: &mut App) {
    app.add_systems(PreStartup, load_resources);
}

pub(crate) fn load_resources(
    mut cmd: Commands,
    mut assets: ResMut<AssetServer>,
    mut resources: ResMut<GameResourceHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut load_material = |name: MaterialName, image: String| {
        let texture_handle: Handle<Image> = assets.load(image);

        let added_material = assets.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(texture_handle),
            perceptual_roughness: 0.85,
            metallic: 0.01,
            unlit: false,
            ..default()
        });

        resources.materials.insert(name, added_material);
    };

    // Materials
    load_material(MaterialName::Cobble, ez_str("cobble.png"));
    load_material(MaterialName::MossyCobble, ez_str("mossy_cobble.png"));
    load_material(MaterialName::Brick, ez_str("brick.png"));
    load_material(MaterialName::RoughStone, ez_str("rough_stone.png"));

    load_material(MaterialName::Dice, ez_str("cardsMedium_tilemap_packed.png"));

    // Meshes
    resources.cube = meshes.add(Cuboid {
        half_size: Vec3::splat(TILE_SIZE / 2.0),
    });
    resources.plane = meshes.add(Plane3d::default().mesh().size(TILE_SIZE, TILE_SIZE));
    resources.dice_mesh = assets.load("meshes/dice.obj");

    // Fonts
    resources.font = assets.load("fonts/Minecraftchmc-dBlX.ttf");

    // TODO: Sprites
    let mut dict: HashMap<EnemyKind, Handle<Image>> = HashMap::new();
    dict.insert(EnemyKind::Skull, assets.load("enemy_sprites/skull.png"));
    resources.enemy_sprites = dict;

    {
        let size = Extent3d {
            width: 160 * 3,
            height: 120 * 3,
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
        resources.render_texture = images.add(render_texture);
    }
}

#[derive(Resource, Default)]
pub struct UserSettings {
    pub mouse_sens: f32,
}
