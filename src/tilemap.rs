use crate::{
    enemy::{EnemyKind, SpawnEnemyEvent},
    mathx::*,
};
use bevy::{
    math::{vec2, vec3, I64Vec2},
    prelude::*,
    render::mesh,
};
use bevy_rapier3d::prelude::*;
use tiled::Loader;

use crate::GameResourceHandles;

const CHUNK_SIZE: i32 = 8;
const TILE_SIZE_PIXELS: i32 = 72;
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

pub fn pixels_to_world(pix_x: f32, pix_y: f32) -> Vec3 {
    let conv_x = (pix_x / TILE_SIZE_PIXELS as f32) * TILE_SIZE - TILE_SIZE / 2.0;
    let conv_y = (pix_y / TILE_SIZE_PIXELS as f32) * TILE_SIZE - TILE_SIZE / 2.0;

    vec3(conv_x, -TILE_SIZE / 2.0, conv_y)
}

#[derive(Event)]
pub struct CreateTilemapEvent;

pub(crate) fn create_tilemap_listener(
    mut events: EventReader<CreateTilemapEvent>,
    mut enemy_events: EventWriter<SpawnEnemyEvent>,
    mut commands: Commands,
    resources: Res<GameResourceHandles>,
) {
    if events.is_empty() {
        return;
    }

    for ev in events.read() {
        let mut loader = Loader::new();
        let map = match loader.load_tmx_map("assets/map.tmx") {
            Ok(map) => map,
            Err(_) => return,
        };

        let enemy_layer = map
            .layers()
            .find(|l| l.name == "Tile_Enemy")
            .map(|layer| layer.as_object_layer())
            .unwrap_or_else(|| todo!());

        if let Some(object) = enemy_layer {
            for obj in object.object_data() {
                let converted_pos = pixels_to_world(obj.x, obj.y);
                println!("{:?}", converted_pos);

                enemy_events.send(SpawnEnemyEvent {
                    position: converted_pos,
                    kind: EnemyKind::Skull,
                });
                println!("{:?}", obj.name);
                println!(
                    "{:?}",
                    vec3(obj.x as f32 / TILE_SIZE, 3.0, obj.y as f32 / TILE_SIZE)
                );
            }
        }

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
                                            crate::mathx::f32::degrees_to_radians(180.0),
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
}
