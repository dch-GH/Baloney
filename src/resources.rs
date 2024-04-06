use bevy::{
    prelude::*,
    render::{camera::PhysicalCameraParameters, mesh::morph::MeshMorphWeights, texture::*},
};

use crate::TILE_SIZE;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraParameters(pub PhysicalCameraParameters);

#[derive(Resource, Default)]
pub struct GameResourceHandles {
    pub floor_material: Handle<StandardMaterial>,
    pub ceiling_material: Handle<StandardMaterial>,
    pub wall_material: Handle<StandardMaterial>,
    pub cube: Handle<Mesh>,
    pub plane: Handle<Mesh>,
}

pub fn init_resources(
    mut cmd: Commands,
    mut assets: ResMut<AssetServer>,
    mut resources: ResMut<GameResourceHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
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

    resources.ceiling_material = ceiling;
    resources.wall_material = wall;
    resources.floor_material = floor;
    resources.cube = cube_handle;
    resources.plane = plane_handle;
}
