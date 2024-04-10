use bevy::{
    app::AppExit,
    ecs::prelude::*,
    input::{keyboard::KeyCode, ButtonInput},
    math::{vec2, Vec2},
    window::{CursorGrabMode, Window, WindowMode},
};

use crate::{MouseLookEnabled, Player};

pub(crate) fn window_start(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
}

pub(crate) fn window_update(
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    mut player: Query<Entity, (With<Player>, Without<Window>)>,
    key: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    let mut window = windows.single_mut();

    if player.is_empty() {
        return;
    }
    let mut player = player.single_mut();

    if key.just_pressed(KeyCode::Tab) {
        if window.cursor.visible {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            commands.entity(player).remove::<MouseLookEnabled>();
        } else {
            window.cursor.grab_mode = CursorGrabMode::None;
            let cursor_start = Some(vec2(
                window.resolution.width() / 2.0,
                window.resolution.height() / 2.0,
            ));
            window.set_cursor_position(cursor_start);
            commands.entity(player).insert(MouseLookEnabled);
        }

        window.cursor.visible = !window.cursor.visible;
    }

    if key.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }

    if key.just_pressed(KeyCode::F11) {
        match window.mode {
            WindowMode::Fullscreen => window.mode = WindowMode::Windowed,
            _ => window.mode = WindowMode::Fullscreen,
        }
    }
}
