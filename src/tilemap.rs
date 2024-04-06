use crate::mathx::*;
use bevy::{math::vec3, prelude::*, render::mesh};
use bevy_rapier3d::prelude::*;
use tiled::Loader;

use crate::GameResourceHandles;

const CHUNK_SIZE: i32 = 8;
pub const TILE_SIZE: f32 = 4.0;

enum Tile {
    Air,
    StoneFloor,
    StoneWall,
}

#[derive(Component)]
pub struct TileMap {
    tiles: Vec<tiled::TileId>,
}

pub(crate) fn create_tilemap(commands: &mut Commands, resources: Res<GameResourceHandles>) -> () {
    let mut loader = Loader::new();
    let map = match loader.load_tmx_map("assets/map.tmx") {
        Ok(map) => map,
        Err(_) => return,
    };

    let layer = match map.layers().next() {
        Some(layer) => layer,
        None => return,
    };

    let tile_layer = match layer.as_tile_layer() {
        None => return,
        Some(tiled::TileLayer::Infinite(_)) => return,
        Some(tiled::TileLayer::Finite(tile_layer)) => tile_layer,
    };

    let mut tm = TileMap { tiles: Vec::new() };
    for x in 0..tile_layer.width() {
        for y in 0..tile_layer.height() {
            if let Some(t) = tile_layer.get_tile(x as i32, y as i32) {
                let tile_id = t.id();
                tm.tiles.push(tile_id);

                match tile_id {
                    // Floor
                    0 => {
                        commands
                            .spawn(PbrBundle {
                                mesh: resources.plane.clone(),
                                material: resources.floor_material.clone(),
                                transform: Transform::IDENTITY.with_translation(vec3(
                                    x as f32 * TILE_SIZE,
                                    -TILE_SIZE * 1.5,
                                    y as f32 * TILE_SIZE,
                                )),
                                ..default()
                            })
                            .insert(RigidBody::Fixed)
                            .insert(Collider::cuboid(TILE_SIZE / 2.0, 0.1, TILE_SIZE / 2.0));

                        // Ceiling
                        commands
                            .spawn(PbrBundle {
                                mesh: resources.plane.clone(),
                                material: resources.ceiling_material.clone(),
                                transform: Transform::IDENTITY
                                    .with_translation(vec3(
                                        x as f32 * TILE_SIZE,
                                        -2.0,
                                        y as f32 * TILE_SIZE,
                                    ))
                                    .with_rotation(Quat::from_euler(
                                        EulerRot::XYZ,
                                        degrees_to_radians_f32(180.0),
                                        0.0,
                                        0.0,
                                    )),
                                ..default()
                            })
                            .insert(RigidBody::Fixed)
                            .insert(Collider::cuboid(TILE_SIZE / 2.0, 0.1, TILE_SIZE / 2.0));
                    }
                    // Wall
                    1 => {
                        commands
                            .spawn(PbrBundle {
                                mesh: resources.cube.clone(),
                                material: resources.wall_material.clone(),
                                transform: Transform::IDENTITY.with_translation(vec3(
                                    x as f32 * TILE_SIZE,
                                    -TILE_SIZE,
                                    y as f32 * TILE_SIZE,
                                )),
                                ..default()
                            })
                            .insert(RigidBody::Fixed)
                            .insert(Collider::cuboid(
                                TILE_SIZE / 2.0,
                                TILE_SIZE / 2.0,
                                TILE_SIZE / 2.0,
                            ));
                    }
                    _ => {}
                };
            }
        }
    }

    commands.spawn_empty().insert(tm);
}
