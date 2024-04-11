use std::{borrow::Borrow, collections::HashSet, ops::Mul, thread::panicking};

use bevy::{app::FixedMain, input::mouse::MouseMotion, math::*, prelude::*};

use bevy_rapier3d::{
    control::{self, KinematicCharacterController},
    dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Sleeping, Velocity},
    geometry::{ActiveEvents, Collider},
    na::Dynamic,
    pipeline::CollisionEvent,
    rapier::dynamics::BodyPair,
};

use crate::{
    enemy::Enemy, mathx, GameResourceHandles, LowResCamera, MainCamera, MaterialName, UserSettings,
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Eye {
    pub view: Quat,
    pub pitch: f32,
    pub yaw: f32,
}

impl Eye {
    /// Get the unit Vec3or in the local `X` direction.
    #[inline]
    pub fn local_x(&self) -> Direction3d {
        // Direction3d::new(x) panics if x is of invalid length, but quat * unit Vec3or is length 1
        Direction3d::new(self.view * Vec3::X).unwrap()
    }

    /// Equivalent to [`-local_x()`][Transform::local_x()]
    #[inline]
    pub fn left(&self) -> Direction3d {
        -self.local_x()
    }

    /// Equivalent to [`local_x()`][Transform::local_x()]
    #[inline]
    pub fn right(&self) -> Direction3d {
        self.local_x()
    }

    /// Get the unit Vec3or in the local `Y` direction.
    #[inline]
    pub fn local_y(&self) -> Direction3d {
        // Direction3d::new(x) panics if x is of invalid length, but quat * unit Vec3or is length 1
        Direction3d::new(self.view * Vec3::Y).unwrap()
    }

    /// Equivalent to [`local_y()`][Transform::local_y]
    #[inline]
    pub fn up(&self) -> Direction3d {
        self.local_y()
    }

    /// Equivalent to [`-local_y()`][Transform::local_y]
    #[inline]
    pub fn down(&self) -> Direction3d {
        -self.local_y()
    }

    /// Get the unit Vec3or in the local `Z` direction.
    #[inline]
    pub fn local_z(&self) -> Direction3d {
        // Direction3d::new(x) panics if x is of invalid length, but quat * unit Vec3or is length 1
        Direction3d::new(self.view * Vec3::Z).unwrap()
    }

    /// Equivalent to [`-local_z()`][Transform::local_z]
    #[inline]
    pub fn forward(&self) -> Direction3d {
        -self.local_z()
    }

    /// Equivalent to [`local_z()`][Transform::local_z]
    #[inline]
    pub fn back(&self) -> Direction3d {
        self.local_z()
    }
}

#[derive(Component)]
pub struct CursorUnlocked;

#[derive(Component)]
pub struct Noclip;

#[derive(Component)]
pub struct MoveFlags {
    pub floating: bool,
}

impl Default for MoveFlags {
    fn default() -> Self {
        Self { floating: false }
    }
}

#[derive(Component)]
pub struct Dice {
    rolled: bool,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub eye: Eye,
    pub move_flags: MoveFlags,
    pub controller: bevy_rapier3d::control::KinematicCharacterController,
    pub collider: Collider,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            eye: Eye {
                view: Quat::IDENTITY,
                pitch: 0.0,
                yaw: 0.0,
            },
            move_flags: MoveFlags::default(),
            controller: bevy_rapier3d::control::KinematicCharacterController {
                apply_impulse_to_dynamic_bodies: true,
                custom_mass: Some(1.0),
                ..default()
            },
            collider: Collider::capsule_y(0.885, 0.25),
        }
    }
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
            .insert(PlayerBundle { ..default() });

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
    cam_xform.translation = player_xform.translation + eye_offset;
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

    if key.just_pressed(KeyCode::Space) {
        // Spawn dice
        let mut inherit_velocity = velocity;
        inherit_velocity.y = 0.10;
        // println!("{:?}", player_xform.translation);
        let eye_pos = cam_xform.translation + fwd * 1.5;
        commands
            .spawn(PbrBundle {
                mesh: resources.dice_mesh.clone(),
                material: resources.get_material(MaterialName::Dice),
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
