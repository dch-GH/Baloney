use bevy::{
    ecs::{component::Component, event::EventReader, query::QueryData},
    input::mouse::MouseMotion,
    prelude::*,
};
use bevy_rapier3d::control::KinematicCharacterController;

use crate::{LowResCamera, MainCamera};

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Jumper {
    pub jump_time: f32,
}

pub fn move_player(
    mut query: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &mut Jumper,
        ),
        With<Player>,
    >,
    mut cam_query: Query<
        &mut Transform,
        (With<LowResCamera>, Without<Player>, Without<MainCamera>),
    >,
    mut light_query: Query<
        &mut Transform,
        (
            With<PointLight>,
            Without<Player>,
            Without<MainCamera>,
            Without<LowResCamera>,
        ),
    >,
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

    cam_xform.translation = player_xform.translation
        + Vec3 {
            x: 0.0,
            y: 0.7,
            z: 0.0,
        };
    controller.translation = Some(velocity);

    for ev in mouse_motion_events.read() {
        // println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
        cam_xform.rotation *= Quat::from_euler(EulerRot::XYZ, 0.0, ev.delta.x * -0.015, 0.0);
    }
    // println!("{}", controller.translation.unwrap());

    if key.pressed(KeyCode::ArrowLeft) {
        cam_xform.rotate_y(2.0 * dt);
        println!("{:?}", player_xform.translation);
    } else if key.pressed(KeyCode::ArrowRight) {
        cam_xform.rotate_y(-2.0 * dt);
    }

    pl_xform.translation = player_xform.translation;
}
