use bevy::{
    app::*,
    color::{palettes::tailwind, Color},
    ecs::{
        event::{Event, EventReader},
        system::Commands,
    },
    math::{vec3, Dir3, Vec3},
    pbr::{PointLight, PointLightBundle},
    prelude::TransformBundle,
    transform::components::{GlobalTransform, Transform},
};

use crate::{
    components::{PlayerBundle, PlayerLight},
    player::systems::*,
};

#[derive(Event)]
pub struct SpawnPlayerEvent;

#[derive(Event)]
pub struct DiceRollEvent {
    pub position: Vec3,
}

pub(crate) fn spawn_player_listener(
    mut commands: Commands,
    mut events: EventReader<SpawnPlayerEvent>,
) {
    for ev in events.read() {
        println!("Spawned Player");
        commands.spawn(PointLightBundle {
            transform: Transform::IDENTITY
                .with_translation(Vec3::X * 3.0 + Vec3::Z * 16.0 + Vec3::Y * 1.2),
            point_light: PointLight {
                intensity: 20_000.0,
                shadows_enabled: (false),
                range: 32.0,
                ..PointLight::default()
            },
            ..PointLightBundle::default()
        });

        commands
            .spawn(TransformBundle {
                local: Transform::IDENTITY.with_translation(vec3(4.0, 5.0, 4.0)),
                global: GlobalTransform::IDENTITY,
            })
            .insert(PlayerBundle::default());

        // RPG light
        commands
            .spawn(PointLightBundle {
                transform: Transform::IDENTITY.with_translation(Dir3::Y * 64.0),
                point_light: PointLight {
                    intensity: 20_000.0,
                    shadows_enabled: (false),
                    range: 32.0,
                    ..PointLight::default()
                },
                ..PointLightBundle::default()
            })
            .insert(PlayerLight);
    }
}
