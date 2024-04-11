use bevy::ecs::{bundle::Bundle, component::Component};
use bevy::math::primitives::Direction3d;
use bevy::math::{Quat, Vec3};
use bevy_rapier3d::control::*;
use bevy_rapier3d::dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Velocity};
use bevy_rapier3d::geometry::{ActiveEvents, Collider};
use bevy_rapier3d::plugin::systems::RigidBodyWritebackComponents;

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
            player: Player::default(),
            eye: Eye::default(),
            move_flags: MoveFlags::default(),
            controller: bevy_rapier3d::control::KinematicCharacterController {
                apply_impulse_to_dynamic_bodies: true,
                custom_mass: Some(1.0),
                ..KinematicCharacterController::default()
            },
            collider: Collider::capsule_y(0.885, 0.25),
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub velocity: Vec3,
    pub dice_active: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            dice_active: false,
        }
    }
}

#[derive(Component)]
pub struct PlayerLight;

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
    pub rolled: bool,
}

impl Default for Dice {
    fn default() -> Self {
        Self { rolled: false }
    }
}

#[derive(Bundle)]
pub struct DiceBundle {
    pub dice: Dice,
    pub rb: RigidBody,
    pub collider: Collider,
    pub velocity: Velocity,
    pub mass: AdditionalMassProperties,
    pub collision_events: ActiveEvents,
}

impl Default for DiceBundle {
    fn default() -> Self {
        Self {
            dice: Dice::default(),
            rb: RigidBody::Dynamic,
            collider: Collider::cuboid(0.25, 0.25, 0.25),
            velocity: Velocity::default(),
            mass: AdditionalMassProperties::Mass(9.0),
            collision_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

#[derive(Component)]
pub struct DiceCamera;

#[derive(Component)]
pub struct Eye {
    pub view: Quat,
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
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

impl Default for Eye {
    fn default() -> Self {
        Self {
            view: Quat::IDENTITY,
            pitch: 0.0,
            yaw: 0.0,
            position: Vec3::ZERO,
        }
    }
}
