use bevy::{
    ecs::prelude::*,
    input::{keyboard::KeyCode, ButtonInput},
    window::Window,
};

pub(crate) fn window_start(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
}

pub(crate) fn window_update(
    mut windows: Query<&mut Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();
    if keyboard_input.just_pressed(KeyCode::Tab) {
        window.cursor.visible = !window.cursor.visible;
    }
}
