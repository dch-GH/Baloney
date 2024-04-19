use bevy::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dParams};

#[derive(Event)]
pub struct CreateSprite3dEvent {
    pub entity: Entity,
    pub position: Vec3,
    pub image: Handle<Image>,
}

pub(crate) fn init(mut app: &mut App) {
    app.add_systems(FixedFirst, create_sprite_listener);
}

fn create_sprite_listener(
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    mut events: EventReader<CreateSprite3dEvent>,
) {
    for ev in events.read() {
        if let Some(mut entity) = commands.get_entity(ev.entity) {
            entity.insert(
                Sprite3d {
                    image: ev.image.clone(),
                    pixels_per_metre: 20.0,
                    transform: Transform::IDENTITY.with_translation(ev.position),
                    ..default()
                }
                .bundle(&mut sprite_params),
            );
        }
    }
}
