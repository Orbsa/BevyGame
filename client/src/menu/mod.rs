use self::systems::{check_autoconnect_cli, check_is_next_to_button, menu_select};
use crate::{
    despawn_all_component,
    player::{animations, spawn_player_sprite},
    states::GameState,
};
use bevy::prelude::*;

mod scene;
mod systems;

pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuItem;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::MainMenu),
            (
                scene::spawn_menu_scene,
                spawn_player_sprite,
                check_autoconnect_cli,
            ),
        )
        .add_systems(Update, animations)
        .add_systems(
            Update,
            (menu_select, check_is_next_to_button)
                .distributive_run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(
            OnExit(GameState::MainMenu),
            despawn_all_component::<MenuItem>,
        );
    }
}
