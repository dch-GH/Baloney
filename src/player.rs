pub mod components;
pub mod events;
pub mod systems;

use bevy::app::App;
use bevy::app::FixedMain;
use bevy::app::Update;

use self::events::*;
use self::systems::*;

pub fn init(mut app: &mut App) {
    app.add_event::<SpawnPlayerEvent>();
    app.add_event::<DiceRollEvent>();

    app.add_systems(FixedMain, spawn_player_listener);
    app.add_systems(Update, (move_player, move_light, dice_system));
}
