use bevy::{
    app::*,
    ecs::{
        event::{Event, EventReader},
        system::Commands,
    },
    math::{primitives::Direction3d, vec3},
    pbr::{PointLight, PointLightBundle},
    render::color::Color,
    transform::{
        components::{GlobalTransform, Transform},
        TransformBundle,
    },
};

use crate::{components::PlayerBundle, player::systems::*};

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
            .insert(PlayerBundle::default());

        // RPG light
        commands.spawn(PointLightBundle {
            transform: Transform::IDENTITY.with_translation(Direction3d::Y * 64.0),
            point_light: PointLight {
                intensity: 20_000.0,
                color: Color::ANTIQUE_WHITE,
                shadows_enabled: (false),
                range: 32.0,
                ..PointLight::default()
            },
            ..PointLightBundle::default()
        });
    }
}
