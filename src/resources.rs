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

use crate::tilemap::TILE_SIZE;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraParameters(pub PhysicalCameraParameters);

#[derive(Resource, Default)]
pub struct GameResourceHandles {
    pub floor_material: Handle<StandardMaterial>,
    pub ceiling_material: Handle<StandardMaterial>,
    pub wall_material: Handle<StandardMaterial>,
    pub cube: Handle<Mesh>,
    pub plane: Handle<Mesh>,
    pub render_texture: Handle<Image>,
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

    let ceiling = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(cobble_texture_handle.clone()),

        unlit: false,
        ..default()
    });

    let floor = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(mossy_cobble_texture_handle.clone()),

        unlit: false,
        ..default()
    });

    let wall = assets.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(brick_texture_handle.clone()),

        unlit: false,
        ..default()
    });

    let cube_handle = meshes.add(Cuboid {
        half_size: Vec3::ONE * 2.0,
    });

    let plane_handle = meshes.add(Plane3d::default().mesh().size(TILE_SIZE, TILE_SIZE));

    let size = Extent3d {
        width: 160,
        height: 120,
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

    resources.ceiling_material = ceiling;
    resources.wall_material = wall;
    resources.floor_material = floor;
    resources.cube = cube_handle;
    resources.plane = plane_handle;
    resources.render_texture = render_texture_handle;
}
