use bevy::{math::vec3, prelude::*, render::mesh};
use bevy_rapier3d::prelude::*;
use tiled::Loader;

const CHUNK_SIZE: i32 = 8;

enum Tile {
    Air,
    StoneFloor,
    StoneWall,
}

#[derive(Component)]
pub struct TileMap {
    tiles: Vec<tiled::TileId>,
}

pub(crate) fn create_tilemap(
    commands: &mut Commands,
    mesh_h: &Handle<Mesh>,
    mat_h: &Handle<StandardMaterial>,
) -> () {
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
                    0 => {}
                    1 => {
                        commands
                            .spawn(PbrBundle {
                                mesh: mesh_h.clone(),
                                material: mat_h.clone(),
                                transform: Transform::IDENTITY.with_translation(vec3(
                                    x as f32 * 4.0,
                                    -4.0,
                                    y as f32 * 4.0,
                                )),
                                ..default()
                            })
                            .insert(RigidBody::Fixed)
                            .insert(Collider::cuboid(1.0, 1.0, 1.0));
                    }
                    _ => {}
                };
            }
        }
    }

    commands.spawn_empty().insert(tm);
}
