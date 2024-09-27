use std::time::Duration;

use bevy::prelude::*;
use shared::{
    casting::DespawnTime,
    event::{client::Chat, server::SendChat, NetEntId, ERFE},
    netlib::{
        send_event_to_server, EventToClient, EventToServer, MainServerEndpoint, ServerResources,
    },
    AnyUnit,
};

use crate::{player::PlayerName, states::GameState};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ChatState {
    #[default]
    NotChatting,
    Chatting,
}

/// chat is this real?
pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ChatState>()
            .add_event::<Chat>()
            .add_event::<WeChat>()
            .insert_resource(ChatHistory::default())
            .insert_resource(ChatHistoryPtr::default())
            .add_systems(Startup, setup_panel)
            .add_systems(Update, on_chat)
            .add_systems(Update, on_chat_type.run_if(in_state(ChatState::Chatting)))
            .add_systems(
                Update,
                on_chat_toggle.run_if(shared::GameAction::Chat.just_pressed()),
            )
            .add_systems(
                Update,
                (receive_network_chats, on_local_chat_send)
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

#[derive(Component)]
struct ChatContainer;

/// This is the box that you type into
#[derive(Component)]
struct ChatTypeContainer;

trait EditableText {
    fn get_text(&mut self) -> &mut String;
}

impl EditableText for &mut Text {
    fn get_text(&mut self) -> &mut String {
        &mut self.sections[0].value
    }
}

#[derive(Resource, Debug, Default)]
struct ChatHistory(Vec<String>);

/// If you press the up arrow, we need to remember which chats you have seen.
#[derive(Resource, Debug, Default)]
struct ChatHistoryPtr(Option<usize>);


fn on_chat_toggle(
    cur_chat_state: Res<State<ChatState>>,
    mut chat_state: ResMut<NextState<ChatState>>,
    mut typed_text: Query<&mut Text, With<ChatTypeContainer>>,
    mut chat_bg_color: Query<&mut BackgroundColor, With<ChatContainer>>,
    mut chat_history: ResMut<ChatHistory>,
    mut chat_history_ptr: ResMut<ChatHistoryPtr>,
    mut ew: EventWriter<WeChat>,
) {
    let mut chatbox = typed_text.single_mut();
    let mut chatbox = chatbox.as_mut();
    let cur_text: &mut String = chatbox.get_text();
    match cur_chat_state.get() {
        ChatState::Chatting => {
            let chat = std::mem::take(cur_text);
            if !chat.is_empty() {
                // If this is a new chat, push it to the front
                if chat_history.0.last() != Some(&chat) {
                    chat_history.0.push(chat.clone());
                }

                ew.send(WeChat(chat));
            }

            *chat_history_ptr = ChatHistoryPtr(None);
            chat_state.set(ChatState::NotChatting);
            *chat_bg_color.single_mut() = Color::WHITE.with_a(0.00).into();
        }
        ChatState::NotChatting => {
            *cur_text = "".into();
            chat_state.set(ChatState::Chatting);
            *chat_bg_color.single_mut() = Color::WHITE.with_a(1.00).into();
        }
    }
}

#[derive(Event, Debug)]
struct WeChat(String);

fn on_chat_type(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut typed_chars: EventReader<ReceivedCharacter>,
    mut typed_text: Query<&mut Text, With<ChatTypeContainer>>,
    mut chat_history_ptr: ResMut<ChatHistoryPtr>,
    chat_history: Res<ChatHistory>,
) {
    let mut chatbox = typed_text.single_mut();
    let mut chatbox = chatbox.as_mut();
    let cur_text = chatbox.get_text();

    if keyboard_input.just_pressed(KeyCode::Backspace) {
        cur_text.pop();
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) && !chat_history.0.is_empty() {
        let new_ptr = chat_history_ptr
            .0
            .unwrap_or(chat_history.0.len())
            .saturating_sub(1);

        *chat_history_ptr = ChatHistoryPtr(Some(new_ptr));
        *cur_text = chat_history.0[new_ptr].clone();
    }

    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        if let Some(cur_ptr) = chat_history_ptr.0 {
            let new_ptr = (cur_ptr + 1).min(chat_history.0.len() - 1);

            *chat_history_ptr = ChatHistoryPtr(Some(new_ptr));
            *cur_text = chat_history.0[new_ptr].clone();
        };
    }

    for typed_char in typed_chars.read() {
        for c in typed_char.char.chars() {
            if !c.is_control() {
                cur_text.push(c);
            }
        }
    }
}

fn setup_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    // setup a flexbox container for notifications
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(400.0),
                    height: Val::Px(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::WHITE.with_a(0.).into(),
                ..default()
            },
            UiImage::new(asset_server.load("textures/Chat_RS.png")),
            // TODO: Adjust base asset here to be  overlay, and some tileable paper texture underneath
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::rectangle(16., 22.),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 4.0},
                // we don't stretch the corners more than their actual size (20px)
                max_corner_scale: 1.0,
                ..default()
            }),
            ChatContainer,
        ))
        .with_children(|parent| {
            // This is the box to type into
            parent.spawn((
                TextBundle::from_sections([TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/ttf/Runescape-Bold-12.ttf"),
                        font_size: 12.0,
                        color: Color::BLACK,
                    },
                )])
                .with_text_justify(JustifyText::Left)
                .with_style(Style { position_type: PositionType::Absolute, bottom : Val::Px(6.) , left: Val::Px(32.), ..default() }),
                ChatTypeContainer,
            ));
        });
}

fn receive_network_chats(mut net_chats: ERFE<Chat>, mut ew: EventWriter<Chat>) {
    for c in net_chats.read() {
        ew.send(c.event.clone());
    }
}

fn on_local_chat_send(
    mut er: EventReader<WeChat>,
    //mut ew: EventWriter<Chat>,
    //our_id: Query<&NetEntId, With<Player>>,
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    for e in er.read() {
        let event = EventToServer::SendChat(SendChat { text: e.0.clone() });
        send_event_to_server(&sr.handler, mse.0, &event);
    }
}

fn on_chat(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    chat_box: Query<Entity, With<ChatContainer>>,
    players: Query<(&NetEntId, &PlayerName), With<AnyUnit>>,
    mut er: EventReader<Chat>,
    time: Res<Time>,
) {
    for chat in er.read() {
        let name = match chat.source {
            Some(ref eid) => {
                let matching = players.iter().find(|(ent_id, _)| eid == *ent_id);
                matching
                    .map(|(_, player_name)| player_name.0.as_ref())
                    .unwrap_or("Unknown Player")
            }
            None => "Server",
        };

        let parent = chat_box.single();
        commands.entity(parent).with_children(|p| {
            p.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        format!("{:03.3} ", time.elapsed_seconds()),
                        TextStyle {
                            font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                            font_size: 14.0,
                            color: Color::DARK_GRAY,
                        },
                    ),
                    TextSection::new(
                        name,
                        TextStyle {
                            font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                            font_size: 16.0,
                            color: Color::ORANGE_RED,
                        },
                    ),
                    TextSection::new(
                        ": ".to_string(),
                        TextStyle {
                            font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                            font_size: 16.0,
                            color: Color::GRAY,
                        },
                    ),
                    TextSection::new(
                        &chat.text,
                        TextStyle {
                            font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ),
                ])
                .with_text_justify(JustifyText::Left)
                .with_style(Style { top: Val::Px(20.) , left: Val::Px(20.), ..default() }),
                DespawnTime(Timer::new(Duration::from_millis(15000), TimerMode::Once)),
            ));
        });
    }
}
