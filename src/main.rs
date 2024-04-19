#![allow(warnings)]

mod camera;
mod enemy;
mod mathx;
mod player;
mod resources;
mod sprite;
mod tilemap;
mod ui;
mod utils;
mod windows;

use bevy_sprite3d::Sprite3dPlugin;
use camera::{CameraState, LowResCamera, MainCamera};
use enemy::SpawnEnemyEvent;
use player::*;
use resources::*;
use sprite::CreateSprite3dEvent;
use tilemap::*;
use ui::*;

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
        app.insert_resource(UserSettings { mouse_sens: 0.005 })
            .insert_resource(GameResourceHandles::default());

        app.insert_resource(CameraState::default());
        app.insert_resource(CameraParameters(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
        }));

        app.insert_resource(ui::GameUi { ui_entity: None });
    }

    // Modules init Events, Listeners, and Systems.
    resources::init(&mut app);
    windows::init(&mut app);
    ui::init(&mut app);
    player::init(&mut app);
    tilemap::init(&mut app);
    camera::init(&mut app);
    enemy::init(&mut app);
    sprite::init(&mut app);

    // Systems
    app.add_systems(Startup, start);
    app.add_systems(Update, debug_info);

    app.run();
}

fn start(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resources: Res<GameResourceHandles>,
    cam_parameters: Res<CameraParameters>,
    mut tilemap_event: EventWriter<CreateTilemapEvent>,
    mut spawn_player_event: EventWriter<player::events::SpawnPlayerEvent>,
    mut ui_event: EventWriter<CreateUiEvent>,
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

    ui_event.send(ui::CreateUiEvent {
        camera_entity: main_camera,
    });

    tilemap_event.send(CreateTilemapEvent);
    spawn_player_event.send(player::events::SpawnPlayerEvent);
}

fn debug_info(key: Res<ButtonInput<KeyCode>>, mut physics_debug: ResMut<DebugRenderContext>) {
    if key.just_pressed(KeyCode::KeyZ) {
        physics_debug.enabled = !physics_debug.enabled;
    }
}
