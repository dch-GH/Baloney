use rand::prelude::*;

use crate::{sprite::CreateSprite3dEvent, GameResourceHandles};

use bevy::{
    app::AppExit,
    ecs::{component::Component, event::EventReader, query::QueryData},
    gizmos,
    input::mouse::MouseMotion,
    math::vec3,
    prelude::*,
};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Sleeping, Velocity},
    geometry::Collider,
    na::Dynamic,
    pipeline::CollisionEvent,
    rapier::dynamics::BodyPair,
};
use bevy_sprite3d::{Sprite3d, Sprite3dBundle};

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum EnemyKind {
    Skull,
}

#[derive(Component)]
pub struct Enemy {
    kind: EnemyKind,
}

#[derive(Component, Default)]
pub struct EnemyMotor {
    pub move_dir: Vec3,
    pub time_since_chose_direction: f32,
}

#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub position: Vec3,
    pub kind: EnemyKind,
}

pub fn create_enemy_listener(
    mut commands: Commands,
    resources: Res<GameResourceHandles>,
    mut spawn_events: EventReader<SpawnEnemyEvent>,
    mut event_bus: EventWriter<CreateSprite3dEvent>,
) {
    for ev in spawn_events.read() {
        let pos = ev.position;

        let enemy = commands
            .spawn(TransformBundle {
                local: Transform::IDENTITY.with_translation(pos),
                global: GlobalTransform::IDENTITY,
            })
            .insert(bevy_rapier3d::control::KinematicCharacterController {
                apply_impulse_to_dynamic_bodies: true,
                custom_mass: Some(1.0),
                ..default()
            })
            .insert(Enemy {
                kind: ev.kind.clone(),
            })
            .insert(EnemyMotor { ..default() })
            .insert(Collider::capsule_y(0.6, 1.5))
            .id();

        println!("Spawned lil bro at: {:?}", pos);
        event_bus.send(CreateSprite3dEvent {
            entity: enemy,
            position: pos,
            image: resources.enemy_sprites.get(&ev.kind).unwrap().clone(),
        });
    }
}

pub fn enemy_motor(
    mut query: Query<
        (
            &Transform,
            &mut EnemyMotor,
            &mut KinematicCharacterController,
        ),
        With<Enemy>,
    >,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let dt = time.delta_seconds();

    for (xform, mut motor, mut controller) in query.iter_mut() {
        if motor.time_since_chose_direction >= 3.0 {
            let mut dir = crate::mathx::vector::random::vec3();
            dir.y = xform.translation.y;

            motor.move_dir = dir * 3.0;
            motor.time_since_chose_direction = 0.0;
        }

        motor.time_since_chose_direction += dt;
        let mut velocity = motor.move_dir * 2.0 * dt;
        velocity += Direction3d::NEG_Y * crate::mathx::GRAVITY * dt;
        controller.translation = Some(velocity);

        gizmos.line(
            xform.translation,
            xform.translation + velocity * 5.0,
            Color::WHITE,
        );
    }
}
