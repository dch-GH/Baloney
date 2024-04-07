use bevy::{
    app::AppExit,
    ecs::{component::Component, event::EventReader, query::QueryData},
    input::mouse::MouseMotion,
    math::vec3,
    prelude::*,
};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    dynamics::{AdditionalMassProperties, ExternalImpulse, RigidBody, Sleeping, Velocity},
    geometry::Collider,
    na::Dynamic,
    rapier::dynamics::BodyPair,
};
use bevy_sprite3d::{Sprite3d, Sprite3dBundle};

use crate::{sprite_system::CreateSprite3dEvent, GameResourceHandles};

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum EnemyKind {
    Skull,
}

#[derive(Component)]
pub struct Enemy {
    kind: EnemyKind,
}

#[derive(Event)]
pub struct SpawnEnemyEvent {
    position: Vec3,
    kind: EnemyKind,
}

pub fn create_enemy_listener(
    mut commands: Commands,
    resources: Res<GameResourceHandles>,
    mut spawn_events: EventReader<SpawnEnemyEvent>,
    mut event_bus: EventWriter<CreateSprite3dEvent>,
) {
    if spawn_events.is_empty() {
        return;
    }

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
            .insert(Collider::capsule_y(0.885, 0.25))
            .id();

        event_bus.send(CreateSprite3dEvent {
            entity: enemy,
            position: pos,
            image: resources.enemy_sprites.get(&ev.kind).unwrap().clone(),
        });
    }
}
