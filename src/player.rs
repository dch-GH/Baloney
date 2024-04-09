use std::{borrow::Borrow, collections::HashSet, thread::panicking};

use bevy::{
    app::AppExit,
    ecs::{component::Component, event::EventReader, query::QueryData, system::SystemState},
    input::mouse::MouseMotion,
    math::vec3,
    prelude::*,
};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Sleeping, Velocity},
    geometry::{ActiveEvents, Collider},
    na::Dynamic,
    pipeline::CollisionEvent,
    rapier::dynamics::BodyPair,
};

use crate::{enemy::Enemy, GameResourceHandles, LowResCamera, MainCamera, UserSettings};

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Dice {
    rolled: bool,
}

pub fn subscribe_events(mut app: &mut App) {
    app.add_systems(Update, dice_collision_listener);
}

pub fn move_player(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut KinematicCharacterController), With<Player>>,
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
    resources: Res<GameResourceHandles>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    key: Res<ButtonInput<KeyCode>>,
    user_cfg: Res<UserSettings>,
) {
    let dt = time.delta_seconds();
    let mv_speed = 10.0;

    let (player_xform, mut controller) = query.single_mut();
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

    let eye_offset = Direction3d::Y * 0.7;
    cam_xform.translation = player_xform.translation + eye_offset;

    controller.translation = Some(velocity);

    for ev in mouse_motion_events.read() {
        cam_xform.rotation *= Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            -ev.delta.x * user_cfg.mouse_sens * dt,
            0.0,
        );
    }

    if key.pressed(KeyCode::ArrowLeft) {
        cam_xform.rotate_y(2.0 * dt);
    } else if key.pressed(KeyCode::ArrowRight) {
        cam_xform.rotate_y(-2.0 * dt);
    }

    if key.just_pressed(KeyCode::KeyE) {
        println!("{:?}", player_xform.translation);
    }

    if key.just_pressed(KeyCode::Space) {
        // Spawn dice
        // println!("{:?}", player_xform.translation);
        let eye_pos = cam_xform.translation + fwd * 1.5;
        commands
            .spawn(PbrBundle {
                mesh: resources.dice_mesh.clone(),
                material: resources.dice_material.clone(),
                transform: Transform::IDENTITY.with_translation(eye_pos),
                ..default()
            })
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Dice { rolled: false })
            .insert(RigidBody::Dynamic)
            .insert(Velocity { ..default() })
            .insert(Collider::cuboid(0.25, 0.25, 0.25))
            .insert(AdditionalMassProperties::Mass(9.0))
            .insert(ExternalImpulse {
                impulse: fwd * 0.3,
                torque_impulse: vec3(0.03, -0.02, 0.04),
            });
    }

    pl_xform.translation = player_xform.translation + eye_offset * 0.35;
}

pub fn dice_system(mut query: Query<(&Transform, &bevy_rapier3d::dynamics::Velocity, &mut Dice)>) {
    if query.is_empty() {
        return;
    }
    let x = World::new();

    for (dice_xform, vel, mut dice) in query.iter_mut() {
        if dice.rolled {
            continue;
        }

        if vel.angvel.length() <= 0.1 || vel.linvel.length() <= 0.1 {
            dice.rolled = true;
            // TODO: something like this
            //event.send(DiceRolledEvent{})
        }

        // println!("{:?}", vel.linvel);
    }
}

fn dice_collision_listener(mut commands: Commands, mut events: EventReader<CollisionEvent>) {
    let mut p = |a: Entity, b: Entity| {
        commands.add(move |world: &mut World| {
            let dice_ent = if world.entity(a).contains::<Dice>() {
                a
            } else {
                b
            };

            let enemy_ent = if dice_ent == a { b } else { a };
            world.despawn(dice_ent);
        });
    };

    for ev in events.read() {
        if let CollisionEvent::Started(a, b, flags) = ev {
            p(*a, *b);
        }
    }
}
