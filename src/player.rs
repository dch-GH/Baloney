use std::{borrow::Borrow, collections::HashSet, thread::panicking};

use bevy::{
    app::{AppExit, FixedMain},
    ecs::{component::Component, event::EventReader, query::QueryData, system::SystemState},
    input::mouse::MouseMotion,
    math::vec3,
    prelude::*,
};
use bevy_rapier3d::{
    control::{self, KinematicCharacterController},
    dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Sleeping, Velocity},
    geometry::{ActiveEvents, Collider},
    na::Dynamic,
    pipeline::CollisionEvent,
    rapier::dynamics::BodyPair,
};

use crate::{enemy::Enemy, mathx, GameResourceHandles, LowResCamera, MainCamera, UserSettings};

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct MouseLookEnabled;

#[derive(Component)]
pub struct Noclip;

#[derive(Component)]
pub struct MoveFlags {
    pub floating: bool,
}

#[derive(Component)]
pub struct Dice {
    rolled: bool,
}

#[derive(Event)]
pub struct SpawnPlayerEvent;

pub fn subscribe_events(mut app: &mut App) {
    app.add_event::<SpawnPlayerEvent>();
    app.add_systems(FixedMain, spawn_player_listener);
    app.add_systems(Update, (move_player, dice_system));
}

fn spawn_player_listener(mut commands: Commands, mut events: EventReader<SpawnPlayerEvent>) {
    for ev in events.read() {
        println!("Spawned Player");
        commands
            .spawn(TransformBundle {
                local: Transform::IDENTITY.with_translation(vec3(4.0, 5.0, 4.0)),
                global: GlobalTransform::IDENTITY,
            })
            .insert(bevy_rapier3d::control::KinematicCharacterController {
                apply_impulse_to_dynamic_bodies: true,
                custom_mass: Some(1.0),
                ..default()
            })
            .insert(Player {})
            .insert(MoveFlags { floating: false })
            .insert(Collider::capsule_y(0.885, 0.25));

        // RPG light
        commands.spawn(PointLightBundle {
            transform: Transform::IDENTITY.with_translation(Direction3d::Y * 64.0),
            point_light: PointLight {
                intensity: 20_000.0,
                color: Color::ANTIQUE_WHITE,
                shadows_enabled: (false),
                range: 32.0,
                ..default()
            },
            ..default()
        });
    }
}

pub fn move_player(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut KinematicCharacterController,
            &mut MoveFlags,
            Has<MouseLookEnabled>,
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
) {
    if query.is_empty() {
        return;
    }

    let dt = time.delta_seconds();
    let mv_speed = 10.0;

    let (
        player_entity,
        player_xform,
        mut controller,
        mut move_flags,
        mouse_look_enabled,
        has_noclip,
    ) = query.single_mut();

    let mut cam_xform = cam_query.single_mut();
    let mut pl_xform = light_query.single_mut();

    let mut velocity = Vec3::ZERO;
    let mut wish_move = Vec3::ZERO;
    let mut sprint_mult = 1.0;

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

    if key.pressed(KeyCode::ShiftLeft) {
        sprint_mult = 2.8;
    }

    wish_move = wish_move.normalize_or_zero();
    velocity += wish_move * mv_speed * sprint_mult * dt;

    velocity.x = f32::clamp(velocity.x, -32.0, 32.0);
    velocity.z = f32::clamp(velocity.z, -32.0, 32.0);

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
    cam_xform.translation = player_xform.translation + eye_offset;

    controller.translation = Some(velocity);

    for ev in mouse_motion_events.read() {
        if mouse_look_enabled {
            continue;
        }
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

    if key.just_pressed(KeyCode::Space) {
        // Spawn dice
        let mut inherit_velocity = velocity;
        inherit_velocity.y = 0.10;
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
                // impulse: fwd * 0.3,
                impulse: inherit_velocity * 10.0 + fwd * 0.5,
                torque_impulse: mathx::vector::random::vec3() * 0.5,
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
