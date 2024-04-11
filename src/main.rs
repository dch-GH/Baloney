#![allow(warnings)]

mod enemy;
mod mathx;
mod player;
mod resources;
mod sprite;
mod tilemap;
mod utils;
mod windows;

use bevy_sprite3d::Sprite3dPlugin;
use enemy::{create_enemy_listener, SpawnEnemyEvent};
use player::*;
use resources::*;
use sprite::{create_sprite_listener, CreateSprite3dEvent};
use tilemap::*;

use bevy::{
    asset::{self, LoadState},
    ecs::query::QueryData,
    gltf::Gltf,
    input::{self, keyboard::KeyboardInput, mouse::MouseMotion},
    log::LogPlugin,
    math::vec3,
    prelude::*,
    render::{
        camera::{Exposure, PhysicalCameraParameters, RenderTarget},
        render_resource::{SamplerDescriptor, Texture, TextureId},
        texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    },
    utils::petgraph::{data::Create, visit::Control},
    window::{Cursor, CursorGrabMode},
};

use bevy_obj::ObjPlugin;
use bevy_rapier3d::{control, prelude::*};

#[derive(Component)]
pub struct LowResCamera;

#[derive(Component)]
pub struct MainCamera;

fn main() {
    let mut app = App::new();

    // Plugins
    {
        // TODO: ERROR log: VALIDATION [VUID-VkRenderPassBeginInfo-framebuffer-04627 (0x45125641)]
        app.add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "off".into(),
                    level: bevy::log::Level::ERROR,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Game".into(),
                        cursor: Cursor {
                            grab_mode: CursorGrabMode::Confined,
                            visible: false,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }),
        );
        app.add_plugins(ObjPlugin);
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        });
        app.add_plugins(Sprite3dPlugin);
    }

    // Resources
    {
        app.insert_resource(CameraParameters(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
        }));
        app.insert_resource(UserSettings { mouse_sens: 0.005 })
            .insert_resource(GameResourceHandles::default());
    }

    // Events + Listeners
    {
        app.add_event::<SpawnEnemyEvent>();
        app.add_event::<CreateSprite3dEvent>();

        player::init(&mut app);
        TileMap::init(&mut app);
    }

    // Systems
    {
        app.add_systems(PreStartup, init_resources);
        app.add_systems(Startup, (start, windows::window_start));
        app.add_systems(FixedFirst, (create_sprite_listener, create_enemy_listener));
        app.add_systems(FixedUpdate, (crate::enemy::enemy_motor));
        app.add_systems(Update, (windows::window_update, debug_info));
    }

    app.run();
}

fn start(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resources: Res<GameResourceHandles>,
    cam_parameters: Res<CameraParameters>,
    mut tilemap_event: EventWriter<CreateTilemapEvent>,
    mut spawn_player_event: EventWriter<player::events::SpawnPlayerEvent>,
) {
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.2,
    });

    // Spawn low-res camera
    commands.spawn((
        LowResCamera {},
        Camera3dBundle {
            camera: Camera {
                is_active: true,
                order: -1,
                target: RenderTarget::Image(resources.render_texture.clone()),
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            projection: Projection::Perspective(PerspectiveProjection {
                fov: mathx::f32::degrees_to_radians(90.0),
                ..default()
            }),
            exposure: Exposure::from_physical_camera(**cam_parameters),
            ..default()
        },
        FogSettings {
            color: Color::rgba(0.05, 0.04, 0.08, 1.0),
            falloff: FogFalloff::Linear {
                start: 5.0,
                end: 20.0,
            },
            ..default()
        },
    ));

    // Spawn main camera
    let main_camera = commands
        .spawn((MainCamera {}, Camera2dBundle { ..default() }))
        .id();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            TargetCamera(main_camera),
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                image: UiImage {
                    texture: resources.render_texture.clone(),
                    ..default()
                },
                ..default()
            });
        });

    tilemap_event.send(CreateTilemapEvent);
    spawn_player_event.send(player::events::SpawnPlayerEvent);
}

fn debug_info(key: Res<ButtonInput<KeyCode>>, mut physics_debug: ResMut<DebugRenderContext>) {
    if key.just_pressed(KeyCode::KeyZ) {
        physics_debug.enabled = !physics_debug.enabled;
    }
}
