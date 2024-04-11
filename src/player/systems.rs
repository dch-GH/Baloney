use std::{borrow::Borrow, collections::HashSet, ops::Mul, thread::panicking};

use bevy::{app::FixedMain, input::mouse::MouseMotion, math::*, prelude::*};

use bevy_rapier3d::{
    control::{self, KinematicCharacterController},
    dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Sleeping, Velocity},
    geometry::{ActiveEvents, Collider},
    na::Dynamic,
    parry::query::point,
    pipeline::CollisionEvent,
    rapier::dynamics::BodyPair,
};

use crate::{
    enemy::Enemy, mathx, GameResourceHandles, LowResCamera, MainCamera, MaterialName, UserSettings,
};

use crate::player::components::*;

pub fn move_player(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Player,
            &mut Eye,
            &mut Transform,
            &mut KinematicCharacterController,
            &mut MoveFlags,
            Has<CursorUnlocked>,
            Has<Noclip>,
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
    resources: Res<GameResourceHandles>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    key: Res<ButtonInput<KeyCode>>,
    user_cfg: Res<UserSettings>,
    mut gizmos: Gizmos,
) {
    if query.is_empty() {
        return;
    }

    let dt = time.delta_seconds();
    let mv_speed = 10.0;

    let (
        player_entity,
        mut player,
        mut eye,
        player_xform,
        mut controller,
        mut move_flags,
        cursor_unlocked,
        has_noclip,
    ) = query.single_mut();

    let mut cam_xform = cam_query.single_mut();
    let mut pl_xform = light_query.single_mut();

    let mut velocity = Vec3::ZERO;
    let mut wish_move = Vec3::ZERO;
    let mut sprint_mult = 1.0;

    let up = eye.up();
    let fwd = eye.forward();
    let right = eye.right();

    {
        let fixed_fwd = vec3(fwd.x, 0.0, fwd.z);

        if key.pressed(KeyCode::KeyW) {
            wish_move += fixed_fwd;
        }

        if key.pressed(KeyCode::KeyS) {
            wish_move += -fixed_fwd;
        }

        if key.pressed(KeyCode::KeyD) {
            wish_move += right.xyz();
        }

        if key.pressed(KeyCode::KeyA) {
            wish_move += -right.xyz();
        }

        if key.pressed(KeyCode::ShiftLeft) {
            sprint_mult = 2.8;
        }
    }

    wish_move = wish_move.normalize_or_zero();
    velocity += wish_move * mv_speed * sprint_mult * dt;

    if !has_noclip {
        velocity += Direction3d::NEG_Y * 9.82 * dt;
    } else {
        if key.pressed(KeyCode::KeyR) {
            velocity.y += mv_speed * sprint_mult * dt;
        }

        if key.pressed(KeyCode::KeyF) {
            velocity.y -= mv_speed * sprint_mult * dt;
        }
    }

    let eye_offset = Direction3d::Y * 0.7;
    eye.position = player_xform.translation + eye_offset;
    cam_xform.translation = eye.position;
    player.velocity = velocity;
    controller.translation = Some(velocity);

    let mut mouse_delta = Vec2::ZERO;
    for ev in mouse_motion_events.read() {
        if cursor_unlocked {
            continue;
        }

        mouse_delta = ev.delta * user_cfg.mouse_sens;

        eye.pitch -= mouse_delta.y;
        eye.yaw -= mouse_delta.x;

        eye.pitch = mathx::f32::degrees_to_radians(
            mathx::f32::radians_to_degrees(eye.pitch).clamp(-80.0, 80.0),
        );

        eye.view =
            Quat::from_axis_angle(Vec3::Y, eye.yaw) * Quat::from_axis_angle(Vec3::X, eye.pitch);
    }

    cam_xform.rotation = eye.view;

    let start = cam_xform.translation - up * 1.5;
    gizmos.arrow(start, start + fwd * 5.0, Color::YELLOW);

    if key.just_pressed(KeyCode::KeyE) {
        println!("{:?}", player_xform.translation);
    }

    if key.just_pressed(KeyCode::KeyN) {
        if has_noclip {
            commands.entity(player_entity).remove::<Noclip>();
            move_flags.floating = false;
            controller.filter_flags = bevy_rapier3d::pipeline::QueryFilterFlags::default();
            println!("Noclip OFF");
        } else {
            commands.entity(player_entity).insert(Noclip);
            move_flags.floating = true;
            controller.filter_flags = bevy_rapier3d::pipeline::QueryFilterFlags::all();
            println!("Noclip ON");
        }
    }

    pl_xform.translation = eye.position;
}

pub fn dice_system(
    mut commands: Commands,
    mut query: Query<(&Transform, &bevy_rapier3d::dynamics::Velocity, &mut Dice)>,
    mut player_query: Query<(&Player, &Eye, &Transform), (With<Player>, Without<Dice>)>,
    key: Res<ButtonInput<KeyCode>>,
    resources: Res<GameResourceHandles>,
) {
    if player_query.is_empty() {
        return;
    }

    let (mut player, eye, player_xform) = player_query.single_mut();
    let fwd = eye.forward();
    // Roll the dice
    if key.just_pressed(KeyCode::Space) && !player.dice_active {
        let mut inherit_velocity = player.velocity;
        inherit_velocity.y = 0.10;
        // println!("{:?}", player_xform.translation);
        let spawn_pos = eye.position + fwd * 1.5;
        commands
            .spawn(PbrBundle {
                mesh: resources.dice_mesh.clone(),
                material: resources.get_material(MaterialName::Dice),
                transform: Transform::IDENTITY.with_translation(spawn_pos),
                ..default()
            })
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Dice { rolled: false })
            .insert(RigidBody::Dynamic)
            .insert(Velocity { ..default() })
            .insert(Collider::cuboid(0.25, 0.25, 0.25))
            .insert(AdditionalMassProperties::Mass(9.0))
            .insert(ExternalImpulse {
                // impulse: fwd * 0.3,
                impulse: inherit_velocity * 10.0 + fwd * 0.5,
                torque_impulse: mathx::vector::random::vec3() * 0.5,
            });
    }

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
