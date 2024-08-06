use std::{borrow::Borrow, collections::HashSet, ops::Mul, thread::panicking};

use bevy::prelude::*;
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
use rand::Rng;

use crate::{
    camera::{CameraSceneParams, CameraState},
    enemy::Enemy,
    mathx, AddUiMessageEvent, GameResourceHandles, LowResCamera, MainCamera, MaterialName,
    UserSettings,
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
            With<PlayerLight>,
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
    camera_state: Res<CameraState>,
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
        velocity += Dir3::NEG_Y * 9.82 * dt;
    } else {
        if key.pressed(KeyCode::KeyR) {
            velocity.y += mv_speed * sprint_mult * dt;
        }

        if key.pressed(KeyCode::KeyF) {
            velocity.y -= mv_speed * sprint_mult * dt;
        }
    }

    let eye_offset = Dir3::Y * 0.7;
    eye.position = player_xform.translation + eye_offset;

    if camera_state.scene_params.is_none() {
        cam_xform.translation = eye.position;
    }

    if !player.dice_active {
        player.velocity = velocity;
        controller.translation = Some(velocity);
    }

    if camera_state.scene_params.is_none() {
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
    }

    // let start = cam_xform.translation - up * 1.5;
    // gizmos.arrow(start, start + fwd * 5.0, Color::YELLOW);

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
}

pub(crate) fn move_light(
    mut player_query: Query<&Transform, With<Player>>,
    mut light_query: Query<&mut Transform, (With<PlayerLight>, Without<Player>)>,
    mut time: Res<Time>,
) {
    if player_query.is_empty() || light_query.is_empty() {
        return;
    }

    let player = player_query.single();
    let mut light = light_query.single_mut();

    let dt = time.delta_seconds();

    let target_pos = player.translation + Vec3::Y * 0.5;

    light.translation = Vec3::lerp(light.translation, target_pos, dt * 5.0);
}

pub fn dice_system(
    mut commands: Commands,
    mut query: Query<(&Transform, &bevy_rapier3d::dynamics::Velocity, &mut Dice)>,
    mut player_query: Query<(&mut Player, &Eye, &Transform), (With<Player>, Without<Dice>)>,
    mut camera_query: Query<(&mut Transform), (With<LowResCamera>, Without<Player>, Without<Dice>)>,
    key: Res<ButtonInput<KeyCode>>,
    resources: Res<GameResourceHandles>,
    mut camera_state: ResMut<CameraState>,
    time: Res<Time>,
    mut add_message_event: EventWriter<AddUiMessageEvent>,
) {
    if player_query.is_empty() {
        return;
    }

    let dt = time.delta_seconds();
    let mut rng = rand::thread_rng();

    let (mut player, eye, player_xform) = player_query.single_mut();
    let mut cam_xform = camera_query.single_mut();
    let fwd = eye.forward();

    // Roll the dice
    if key.just_pressed(KeyCode::Space) && !player.dice_active {
        let mut inherit_velocity = player.velocity;
        inherit_velocity.y = 0.07;
        let spawn_pos = eye.position + fwd * 1.5;
        commands
            .spawn(PbrBundle {
                mesh: resources.dice_mesh.clone(),
                material: resources.get_material(MaterialName::Dice),
                transform: Transform::IDENTITY
                    .with_translation(spawn_pos)
                    .with_rotation(mathx::random::quat()),
                ..default()
            })
            .insert(DiceBundle::default())
            .insert(ExternalImpulse {
                impulse: inherit_velocity * rng.gen_range(0.5..6.5) + fwd * 0.5,
                torque_impulse: mathx::random::vec3() * rng.gen_range(0.1..0.3),
            });
        player.dice_active = true;
    }

    if query.is_empty() {
        return;
    }

    let mut get_dice_result = |xform: &Transform| -> i32 {
        let dot_up = xform.up().dot(Vec3::Y);
        let dot_down = xform.down().dot(Vec3::Y);
        let dot_right = xform.right().dot(Vec3::Y);
        let dot_left = xform.left().dot(Vec3::Y);
        let dot_forward = xform.forward().dot(Vec3::Y);
        let dot_backward = xform.back().dot(Vec3::Y);

        if dot_up >= 0.5 {
            return 1;
        }

        if dot_down >= 0.5 {
            return 6;
        }

        if dot_right >= 0.5 {
            return 5;
        }

        if dot_left >= 0.5 {
            return 2;
        }

        if dot_forward >= 0.5 {
            return 3;
        }

        if dot_backward >= 0.5 {
            return 4;
        }

        return 0;
    };

    for (dice_xform, vel, mut dice) in query.iter_mut() {
        if dice.rolled {
            continue;
        }

        if vel.angvel.length() <= 0.1 || vel.linvel.length() <= 0.1 {
            dice.since_landed += dt;
        }

        if dice.since_landed >= 0.2 {
            dice.rolled = true;

            let result = get_dice_result(&dice_xform);
            let message = format!("You rolled a {:?}!", result);
            add_message_event.send(AddUiMessageEvent {
                message,
                duration: 3.0,
            });

            cam_xform.translation =
                dice_xform.translation + Vec3::Y * 0.9 + Vec3::X * 0.8 + Vec3::Z * 0.8;

            let sl_pitch = mathx::f32::degrees_to_radians(-90.0);
            commands.spawn(SpotLightBundle {
                spot_light: SpotLight {
                    color: Srgba::hex("#e6bfaa").unwrap().into(),
                    intensity: 150_000.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::IDENTITY
                    .with_translation(dice_xform.translation + Vec3::Y * 1.4)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, sl_pitch, 0.0, 0.0)),
                ..default()
            });

            camera_state.scene_params = Some(CameraSceneParams {
                target_position: dice_xform.translation,
                pos_offset: Vec3::Y * 1.2,
                duration: 2.0,
            });

            player.dice_active = false;
        }
    }
}
