mod player;
mod tilemap;
mod windows;

use bevy::{
    ecs::query::QueryData,
    input::{self, keyboard::KeyboardInput, mouse::MouseMotion},
    log::LogPlugin,
    prelude::*,
    render::camera::{Exposure, PhysicalCameraParameters},
    utils::petgraph::visit::Control,
};
use bevy_rapier3d::{control, prelude::*};
use tiled::Loader;

use player::*;
use tilemap::*;

#[derive(Resource, Default, Deref, DerefMut)]
struct Parameters(PhysicalCameraParameters);

fn main() {
    App::new()
        // TODO: ERROR log: VALIDATION [VUID-VkRenderPassBeginInfo-framebuffer-04627 (0x45125641)]
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "off".into(),
            level: bevy::log::Level::ERROR,
            ..default()
        }))
        .insert_resource(Parameters(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        // Systems --------------
        .add_systems(Startup, start)
        .add_systems(Update, move_player)
        .add_systems(Startup, windows::window_start)
        .add_systems(Update, windows::window_update)
        .run();
}

fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    parameters: Res<Parameters>,
) {
    let test_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 1.0, 0.0),
        perceptual_roughness: 1.0,
        ..default()
    });

    let test = meshes.add(Cuboid {
        half_size: Vec3::ONE * 2.0,
    });

    create_tilemap(&mut commands, &test, &test_mat);

    // Spawn camera
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 90.0,
                ..default()
            }),
            exposure: Exposure::from_physical_camera(**parameters),
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
    // Plane
    {
        commands
            .spawn(PbrBundle {
                transform: Transform::IDENTITY.with_translation(Direction3d::NEG_Y * 6.0),
                mesh: meshes.add(Plane3d::default().mesh().size(64.0, 64.0)),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 1.0,
                    ..default()
                }),
                ..default()
            })
            .insert(RigidBody::Fixed)
            .insert(Collider::cuboid(64.0, 1.0, 64.0));
    }

    // Create player
    {
        commands
            .spawn(TransformBundle {
                local: Transform::IDENTITY.with_translation(Direction3d::Y * 3.0),
                global: GlobalTransform::IDENTITY,
            })
            .insert(bevy_rapier3d::control::KinematicCharacterController { ..default() })
            .insert(Player {})
            .insert(Jumper { jump_time: 0.0 })
            .insert(Collider::capsule_y(0.885, 0.25));
    }

    // static entity for testing
    commands
        .spawn(PbrBundle {
            mesh: test,
            material: test_mat,
            transform: Transform::IDENTITY,
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(1.0, 1.0, 1.0));

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
}

fn move_player(
    mut query: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &mut Jumper,
        ),
        With<Player>,
    >,
    mut cam_query: Query<(&mut Transform), (With<Camera3d>, Without<Player>)>,
    mut light_query: Query<&mut Transform, (With<PointLight>, Without<Player>, Without<Camera3d>)>,
    key: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let mv_speed = 10.0;

    let (player_xform, mut controller, mut jumper) = query.single_mut();
    let mut cam_xform = cam_query.single_mut();
    let mut pl_xform = light_query.single_mut();

    let mut velocity = Vec3::ZERO;
    let mut wish_move = Vec3::ZERO;
    let fwd = cam_xform.forward().normalize_or_zero();
    let right = cam_xform.right().normalize_or_zero();

    if key.pressed(KeyCode::KeyW) {
        wish_move += fwd;
    }

    if key.pressed(KeyCode::KeyS) {
        wish_move += -fwd;
    }

    if key.pressed(KeyCode::KeyD) {
        wish_move += right;
    }

    if key.pressed(KeyCode::KeyA) {
        wish_move += -right;
    }

    wish_move = wish_move.normalize_or_zero();
    velocity += wish_move * mv_speed * dt;
    velocity.x = f32::clamp(velocity.x, -32.0, 32.0);
    velocity.z = f32::clamp(velocity.z, -32.0, 32.0);
    velocity += Direction3d::NEG_Y * 9.82 * dt;

    const MAX_JUMP_TIME: f32 = 0.13;

    if key.just_released(KeyCode::Space) {
        // velocity += Direction3d::Y * 6.0;
        jumper.jump_time += dt;
    }

    if jumper.jump_time > 0.0 && jumper.jump_time < MAX_JUMP_TIME {
        velocity += Direction3d::Y * 0.15;
        jumper.jump_time += dt;
    }

    if jumper.jump_time >= MAX_JUMP_TIME {
        jumper.jump_time = 0.0
    }

    cam_xform.translation = player_xform.translation;
    controller.translation = Some(velocity);

    for ev in mouse_motion_events.read() {
        // println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
        cam_xform.rotation *= Quat::from_euler(EulerRot::XYZ, 0.0, ev.delta.x * -0.015, 0.0);
    }
    // println!("{}", controller.translation.unwrap());

    if key.pressed(KeyCode::ArrowLeft) {
        cam_xform.rotate_y(2.0 * dt);
    } else if key.pressed(KeyCode::ArrowRight) {
        cam_xform.rotate_y(-2.0 * dt);
    }

    pl_xform.translation = player_xform.translation;
}
