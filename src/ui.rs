use bevy::{color::palettes::tailwind, prelude::*, ui::UiMeta};

use crate::GameResourceHandles;

#[derive(Resource)]
pub struct GameUi {
    pub ui_entity: Option<Entity>,
}

#[derive(Event)]
pub struct CreateUiEvent {
    pub camera_entity: Entity,
}

#[derive(Event)]
pub struct AddUiMessageEvent {
    pub message: String,
    pub duration: f32,
}

#[derive(Component)]
struct UiMessage {
    pub message: String,
    pub duration: f32,
    pub life_time: f32,
    pub added_to_tree: bool,
}

impl Default for UiMessage {
    fn default() -> Self {
        Self {
            message: String::from("A heap of raw iron."),
            duration: 2.0,
            life_time: 0.0,
            added_to_tree: false,
        }
    }
}

impl UiMessage {
    pub(crate) fn with_message(mut self: Self, message: &String) -> Self {
        self.message = String::from(message);
        self
    }
}

pub(crate) fn init(app: &mut App) {
    app.add_event::<CreateUiEvent>();
    app.add_event::<AddUiMessageEvent>();

    app.add_systems(Update, create_ui_listener);
    app.add_systems(Update, ui_message_system);
}

fn create_ui_listener(
    mut commands: Commands,
    resources: Res<GameResourceHandles>,
    mut game_ui: ResMut<GameUi>,
    mut events: EventReader<CreateUiEvent>,
) {
    for ev in events.read() {
        game_ui.ui_entity = Some(
            commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    TargetCamera(ev.camera_entity),
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        image: UiImage {
                            texture: resources.render_texture.clone(),
                            ..default()
                        },
                        ..default()
                    });
                })
                .id(),
        );
    }
}

fn ui_message_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UiMessage)>,
    mut ui: ResMut<GameUi>,
    mut add_ui_message_events: EventReader<AddUiMessageEvent>,
    time: Res<Time>,
    resources: Res<GameResourceHandles>,
) {
    if query.is_empty() == false {
        for (ent, mut msg) in query.iter_mut() {
            if msg.added_to_tree {
                msg.life_time += time.elapsed_seconds();

                if msg.life_time >= msg.duration {
                    commands.entity(ent).remove::<UiMessage>();
                }
            }
        }
    }

    for ev in add_ui_message_events.read() {
        let root = ui.ui_entity.expect("Root UI entity is null this is bad!");
        let text = commands
            .spawn(Text2dBundle {
                text: Text::from_section(
                    &ev.message,
                    TextStyle {
                        font: resources.font.clone(),
                        font_size: 32.0,
                        color: tailwind::YELLOW_100.into(),
                    },
                ),
                ..default()
            })
            .insert(UiMessage::default().with_message(&ev.message))
            .id();

        commands.entity(root).add_child(text);
    }
}
