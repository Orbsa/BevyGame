use std::net::SocketAddr;

use bevy::{ecs::query::QuerySingleError, prelude::*};
use shared::{netlib::NetworkConnectionTarget, Config};

use crate::{cameras::chat::ChatState, cli::CliArgs, player::Player, states::GameState};

use super::scene::{MenuButton, SelectedButton};

pub fn check_autoconnect_cli(
    args: Res<CliArgs>,
    config: Res<Config>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let target = match args.autoconnect.as_deref() {
        None => return,
        Some("") => return,
        Some("main") => NetworkConnectionTarget {
            ip: "john2143.com".into(),
            port: 25565,
        },
        Some("local") => NetworkConnectionTarget {
            ip: config.ip.clone(),
            port: config.port,
        },
        Some(other) => {
            // Split this into ip and port and then connect
            let addr: SocketAddr = other
                .parse()
                .expect("--autoconnect was given an invalid ip and port to connect to");

            NetworkConnectionTarget {
                ip: addr.ip().to_string(),
                port: addr.port(),
            }
        }
    };

    info!(
        ?target,
        "Using --autoconnect command line argument to setup connection."
    );

    commands.insert_resource(target);
    game_state.set(GameState::ClientConnecting);
}

pub fn menu_select(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    _config: Res<shared::Config>,
    mut game_state: ResMut<NextState<GameState>>,
    chat_state: Res<State<ChatState>>,
    buttons: Query<&MenuButton, With<SelectedButton>>,
    config: Res<Config>,
    mut commands: Commands,
) {
    if !config.just_pressed(&keyboard_input, shared::GameAction::Use) || *chat_state.get() == ChatState::Chatting {
        return;
    }

    let button = match buttons.get_single() {
        Ok(button) => button,
        Err(QuerySingleError::NoEntities(_)) => {
            // Play sound error
            info!("Not near a button");
            return;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            info!("Somehow got multiple selected buttons?");
            return;
        }
    };

    info!("Clicked button {button:?}");
    match button {
        MenuButton::Connect => {
            commands.insert_resource(NetworkConnectionTarget {
                ip: "john2143.com".into(),
                port: 25565,
            });
            game_state.set(GameState::ClientSendRequestPacket);
        }
        MenuButton::ConnectLocal => {
            commands.insert_resource(NetworkConnectionTarget {
                ip: config.ip.clone(),
                port: config.port,
            });
            game_state.set(GameState::ClientSendRequestPacket);
        }
        MenuButton::Quit => {
            game_state.set(GameState::Quit);
        }
    }
}

pub fn check_is_next_to_button(
    mut commands: Commands,
    players: Query<(Entity, &Player, &Transform)>,
    mut buttons: Query<
        (Entity, &MenuButton, &mut Transform, Option<&SelectedButton>),
        Without<Player>,
    >,
    time: Res<Time>,
) {
    // This system will add or remove the `SelectedButton` component, and make the buttons spin
    let max_dist = 2.0;
    for (_player_ent, _player, player_transform) in &players {
        for (button_ent, button_type, mut button_transform, selected) in &mut buttons {
            if selected.is_some() {
                button_transform.rotate_x(time.delta_seconds() * 3.0);
                button_transform.rotate_y(time.delta_seconds() * 1.0);
                button_transform.rotate_z(time.delta_seconds() * 1.0);
                let dist = player_transform
                    .translation
                    .distance(button_transform.translation);
                if dist > max_dist {
                    info!("Left range of {button_type:?}");
                    commands.entity(button_ent).remove::<SelectedButton>();
                }
            } else {
                button_transform.rotation = Quat::from_rotation_y(0.0);
                let dist = player_transform
                    .translation
                    .distance(button_transform.translation);
                if dist < max_dist {
                    info!("Approached {button_type:?}");
                    commands.entity(button_ent).insert(SelectedButton);
                }
            }
        }
    }
}
