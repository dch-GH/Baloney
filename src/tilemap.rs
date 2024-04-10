use std::result;

use crate::{
    enemy::{EnemyKind, SpawnEnemyEvent},
    mathx::*,
};
use bevy::{
    math::{vec2, vec3, I64Vec2},
    prelude::*,
    render::mesh,
    utils::tracing::span,
};
use bevy_rapier3d::prelude::*;
use tiled::{FiniteTileLayer, Loader, Map};

use crate::GameResourceHandles;

const FLOOR_LAYER: &str = "Floor";
const WALL_LAYER: &str = "Wall";
const CEILING_LAYER: &str = "Ceiling";

const CHUNK_SIZE: i32 = 8;
const TILE_SIZE_PIXELS: i32 = 16;
pub const TILE_SIZE: f32 = 4.0;

enum Tile {
    Air,
    StoneFloor,
    StoneWall,
}

#[derive(Copy, Clone)]
enum ZLayer {
    Floor,
    Wall,
    Ceiling,
}

#[derive(Component)]
pub struct TileMap {
    floor: Vec<tiled::TileId>,
    wall: Vec<tiled::TileId>,
    ceiling: Vec<tiled::TileId>,
}

#[derive(Event)]
pub struct CreateTilemapEvent;

#[derive(Event)]
pub struct SpawnTileFromIdEvent {
    tile_id: tiled::TileId,
    position: Vec3,
    layer: ZLayer,
}

impl TileMap {
    pub fn pixels_to_world(pix_x: f32, pix_y: f32) -> Vec3 {
        let conv_x = (pix_x / TILE_SIZE_PIXELS as f32) * TILE_SIZE - TILE_SIZE / 2.0;
        let conv_y = (pix_y / TILE_SIZE_PIXELS as f32) * TILE_SIZE - TILE_SIZE / 2.0;

        vec3(conv_x, -TILE_SIZE / 2.0, conv_y)
    }

    fn get_layer<'a>(map: &'a Map, layer_name: &str) -> Option<FiniteTileLayer<'a>> {
        let tile_layer = match map
            .layers()
            .filter(|x| x.name == layer_name)
            .next()
            .unwrap()
            .as_tile_layer()
        {
            Some(tiled::TileLayer::Finite(found_layer)) => found_layer,
            _ => todo!(),
        };

        return Some(tile_layer);
    }

    fn process_tile_layer(
        tm: &mut TileMap,
        tiled_layer: &FiniteTileLayer,
        z_layer: ZLayer,
        event_bus: &mut EventWriter<SpawnTileFromIdEvent>,
    ) -> Vec<tiled::TileId> {
        let mut result_layer = Vec::<tiled::TileId>::new();

        let mut event_batch: Vec<SpawnTileFromIdEvent> = Vec::new();

        for x in 0..8 {
            for y in 0..8 {
                if let Some(t) = tiled_layer.get_tile(x as i32, y as i32) {
                    let tile_id = t.id();
                    println!("X: {}  Y: {} ID: {}", x, y, tile_id);
                    result_layer.push(tile_id);

                    if tile_id <= 1 {
                        println!("air");
                        continue;
                    }

                    let mut position = Vec3 {
                        x: x as f32 * TILE_SIZE,
                        y: 0.0,
                        z: y as f32 * TILE_SIZE,
                    };

                    position.y = match z_layer {
                        ZLayer::Floor => -TILE_SIZE * 2.5,
                        ZLayer::Wall => -TILE_SIZE,
                        ZLayer::Ceiling => -2.0,
                    };

                    event_batch.push(SpawnTileFromIdEvent {
                        tile_id,
                        position: position,
                        layer: z_layer,
                    });
                }
            }
        }

        event_bus.send_batch(event_batch);

        result_layer
    }
}

impl TileMap {
    pub(crate) fn subscribe_events(app: &mut App) {
        app.add_event::<CreateTilemapEvent>();
        app.add_event::<SpawnTileFromIdEvent>();

        app.add_systems(FixedFirst, TileMap::listen_create_tilemap);
        app.add_systems(FixedFirst, TileMap::listen_spawn_tile_from_id);
    }

    pub(crate) fn listen_create_tilemap(
        mut events: EventReader<CreateTilemapEvent>,
        mut enemy_events: EventWriter<SpawnEnemyEvent>,
        mut commands: Commands,
        resources: Res<GameResourceHandles>,
        mut spawn_tile_events: EventWriter<SpawnTileFromIdEvent>,
    ) {
        for ev in events.read() {
            let mut loader = Loader::new();
            let map = match loader.load_tmx_map("assets/map.tmx") {
                Ok(map) => map,
                Err(_) => return,
            };

            let floor = TileMap::get_layer(&map, FLOOR_LAYER);
            let wall = TileMap::get_layer(&map, WALL_LAYER);
            let ceiling = TileMap::get_layer(&map, CEILING_LAYER);

            let mut tm = TileMap {
                floor: Vec::<tiled::TileId>::new(),
                wall: Vec::<tiled::TileId>::new(),
                ceiling: Vec::<tiled::TileId>::new(),
            };

            if let Some(floor) = TileMap::get_layer(&map, FLOOR_LAYER) {
                tm.floor = TileMap::process_tile_layer(
                    &mut tm,
                    &floor,
                    ZLayer::Floor,
                    &mut spawn_tile_events,
                );
            }

            if let Some(wall) = TileMap::get_layer(&map, WALL_LAYER) {
                tm.wall = TileMap::process_tile_layer(
                    &mut tm,
                    &wall,
                    ZLayer::Wall,
                    &mut spawn_tile_events,
                );
            }

            if let Some(ceiling) = TileMap::get_layer(&map, CEILING_LAYER) {
                tm.ceiling = TileMap::process_tile_layer(
                    &mut tm,
                    &ceiling,
                    ZLayer::Ceiling,
                    &mut spawn_tile_events,
                );
            }

            commands.spawn_empty().insert(tm);
        }
    }

    pub fn listen_spawn_tile_from_id(
        mut commands: Commands,
        resources: Res<GameResourceHandles>,
        mut events: EventReader<SpawnTileFromIdEvent>,
    ) {
        for ev in events.read() {
            let x = ev.position.x;
            let y = ev.position.y;
            let z = ev.position.z;

            let mesh = match ev.layer {
                ZLayer::Floor => resources.plane.clone(),
                ZLayer::Wall => resources.cube.clone(),
                ZLayer::Ceiling => resources.plane.clone(),
            };

            let rotation: Quat = match ev.layer {
                ZLayer::Floor => Quat::default(),
                ZLayer::Wall => Quat::default(),
                ZLayer::Ceiling => Quat::from_euler(
                    EulerRot::XYZ,
                    crate::mathx::f32::degrees_to_radians(180.0),
                    0.0,
                    0.0,
                ),
            };

            let collider_size = match ev.layer {
                ZLayer::Floor => vec3(TILE_SIZE / 2.0, 0.1, TILE_SIZE / 2.0),
                ZLayer::Wall => Vec3::splat(TILE_SIZE / 2.0),
                ZLayer::Ceiling => vec3(TILE_SIZE / 2.0, 0.1, TILE_SIZE / 2.0),
            };

            match ev.tile_id {
                // Air
                0 => continue,
                1 => continue,

                // Floor
                2 => {
                    commands
                        .spawn(PbrBundle {
                            mesh: mesh,
                            material: resources.floor_material.clone(),
                            transform: Transform::IDENTITY
                                .with_translation(vec3(
                                    x as f32 * TILE_SIZE,
                                    -TILE_SIZE * 1.5,
                                    y as f32 * TILE_SIZE,
                                ))
                                .with_rotation(rotation),
                            ..default()
                        })
                        .insert(RigidBody::Fixed)
                        .insert(Collider::cuboid(
                            collider_size.x,
                            collider_size.y,
                            collider_size.z,
                        ));
                }

                //Wall
                3 => {
                    commands
                        .spawn(PbrBundle {
                            mesh: resources.cube.clone(),
                            material: resources.wall_material.clone(),
                            transform: Transform::IDENTITY
                                .with_translation(vec3(
                                    x as f32 * TILE_SIZE,
                                    -TILE_SIZE,
                                    y as f32 * TILE_SIZE,
                                ))
                                .with_rotation(rotation),
                            ..default()
                        })
                        .insert(RigidBody::Fixed)
                        .insert(Collider::cuboid(
                            collider_size.x,
                            collider_size.y,
                            collider_size.z,
                        ));
                }
                _ => continue,
            };
        }
    }
}
