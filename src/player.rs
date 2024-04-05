use bevy::{
    ecs::{component::Component, event::EventReader, query::QueryData},
    input::mouse::MouseMotion,
    prelude::*,
};

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Jumper {
    pub jump_time: f32,
}
