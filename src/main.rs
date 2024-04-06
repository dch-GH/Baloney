mod mathx;
mod player;
mod resources;
mod tilemap;
mod windows;

use player::*;
use resources::*;
use tilemap::*;

use bevy::{
    asset::LoadState,
    ecs::query::QueryData,
    input::{self, keyboard::KeyboardInput, mouse::MouseMotion},
    log::LogPlugin,
    math::vec3,
    prelude::*,
    render::{
        camera::{Exposure, PhysicalCameraParameters, RenderTarget},
        render_resource::{SamplerDescriptor, Texture, TextureId},
        texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    },
    utils::petgraph::visit::Control,
};
use bevy_rapier3d::{control, prelude::*};

#[derive(Component)]
pub struct LowResCamera;

#[derive(Component)]
pub struct MainCamera;

fn main() {
    App::new()
        // TODO: ERROR log: VALIDATION [VUID-VkRenderPassBeginInfo-framebuffer-04627 (0x45125641)]
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "off".into(),
                    level: bevy::log::Level::ERROR,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(CameraParameters(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
        }))
        .insert_resource(GameResourceHandles::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(PreStartup, init_resources)
        .add_systems(Startup, start)
        .add_systems(Update, (render_cameras, move_player))
        .add_systems(Startup, windows::window_start)
        .add_systems(Update, (windows::window_update, debug_info))
        .run();
}

fn start(
    mut commands: Commands,
    resources: Res<GameResourceHandles>,
    cam_parameters: Res<CameraParameters>,
) {
    create_tilemap(&mut commands, &resources);

    // Create player
    {
        commands
            .spawn(TransformBundle {
                local: Transform::IDENTITY.with_translation(vec3(4.0, -3.5, 4.0)),
                global: GlobalTransform::IDENTITY,
            })
            .insert(bevy_rapier3d::control::KinematicCharacterController { ..default() })
            .insert(Player {})
            .insert(Jumper { jump_time: 0.0 })
            .insert(Collider::capsule_y(0.885, 0.25));
    }

    // RPG light
    commands.spawn(PointLightBundle {
        transform: Transform::IDENTITY.with_translation(Direction3d::Y * 64.0),
        point_light: PointLight {
            intensity: 20_000.0,
            color: Color::ANTIQUE_WHITE,
            shadows_enabled: (true),
            ..default()
        },
        ..default()
    });

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
                ..default()
            },
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 90.0,
                ..default()
            }),
            exposure: Exposure::from_physical_camera(**cam_parameters),
            ..default()
        },
        FogSettings {
            color: Color::rgba(0.25, 0.25, 0.25, 1.0),
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
}

fn render_cameras(
    mut main_query: Query<&mut Camera, With<MainCamera>>,
    mut low_res_query: Query<&mut Camera, (With<LowResCamera>, Without<MainCamera>)>,
    resources: Res<GameResourceHandles>,
) {
    let mut main = main_query.single_mut();
    let mut low_res = low_res_query.single_mut();
    // low_res.target = RenderTarget::Image(resources.render_texture.clone());
}

fn debug_info(key: Res<ButtonInput<KeyCode>>, mut physics_debug: ResMut<DebugRenderContext>) {
    if key.just_pressed(KeyCode::KeyZ) {
        physics_debug.enabled = !physics_debug.enabled;
    }
}
