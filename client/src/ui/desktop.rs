use bevy::{
    camera::visibility::Visibility,
    color::palettes::{
        css::{DARK_SLATE_GRAY, WHITE},
        tailwind::SLATE_300,
    },
    input_focus::{
        AutoFocus, FocusCause, InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    ui::{
        AlignItems, BackgroundColor, BorderColor, BorderRadius, FlexDirection, Interaction,
        JustifyContent, Node, PositionType, UiRect, Val, percent, px, widget::Text,
    },
    utils::default,
};
use bevy_ecs::{
    children,
    entity::Entity,
    hierarchy::Children,
    query::{Changed, With},
    system::{Commands, Query, Res, ResMut},
};
use bevy_state::state::NextState;
use std::str::FromStr;
use validator::ValidateIp;

use crate::{
    ConnectButton, ConnectPage, HostField, InfoLabel, PlayerName, PlayerNameField, PortField,
    ServerHost, ServerPort, State,
    ui::{HOVERED_BUTTON, NORMAL_BUTTON},
};

pub fn setup_connect_page(
    // mut r_state: ResMut<State>,
    server_host: Res<ServerHost>,
    server_port: Res<ServerPort>,
    player_name: Res<PlayerName>,
    mut state: ResMut<NextState<State>>,
    mut commands: Commands,
) {
    let mut host_editable_text = EditableText::new(server_host.0.to_string().as_str());
    host_editable_text.cursor_width = 0.4;
    host_editable_text.max_characters = Some(62);
    let mut port_editable_text = EditableText::new(server_port.0.to_string().as_str());
    port_editable_text.cursor_width = 0.4;
    port_editable_text.max_characters = Some(5);
    let mut player_name_editable_text = EditableText::new(player_name.0.to_string().as_str());
    player_name_editable_text.cursor_width = 0.4;
    player_name_editable_text.max_characters = Some(20);
    commands
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Visible,
            ConnectPage,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(20.),
                    ..default()
                },
                InfoLabel,
                Text::new(""),
            ));
        });

    commands
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Visible,
            ConnectPage,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.)),
                        row_gap: Val::Px(10.),
                        flex_direction: FlexDirection::Column,
                        border: px(2.).all(),
                        border_radius: BorderRadius::all(Val::Percent(10.)),
                        ..default()
                    },
                    BorderColor::all(bevy::color::palettes::css::WHITE),
                    TabGroup::new(0),
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            column_gap: Val::Px(10.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((Node::default(), Text::new("Host")));
                            parent.spawn((
                                HostField,
                                Node {
                                    width: px(200),
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                host_editable_text,
                                TabIndex(0),
                                TextCursorStyle {
                                    color: bevy_color::Color::Srgba(WHITE),
                                    ..Default::default()
                                },
                                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic()),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                                AutoFocus,
                            ));
                            parent.spawn((Node::default(), Text::new("Port")));
                            parent.spawn((
                                PortField,
                                Node {
                                    width: px(80),
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                port_editable_text,
                                TabIndex(1),
                                TextCursorStyle {
                                    color: bevy_color::Color::Srgba(WHITE),
                                    ..Default::default()
                                },
                                EditableTextFilter::new(|c| {
                                    c.is_ascii() && c.is_ascii_graphic() && c.is_numeric()
                                }),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                            ));
                        });

                    parent
                        .spawn(Node {
                            column_gap: Val::Px(10.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    ..Default::default()
                                },
                                Text::new("Player name"),
                            ));
                            parent.spawn((
                                PlayerNameField,
                                Node {
                                    width: px(100),
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                player_name_editable_text,
                                AutoFocus,
                                TabIndex(2),
                                TextCursorStyle {
                                    color: bevy_color::Color::Srgba(WHITE),
                                    ..Default::default()
                                },
                                EditableTextFilter::new(|c| c.is_ascii_alphabetic()),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                            ));
                            parent.spawn((
                                ConnectButton,
                                Interaction::default(),
                                TabIndex(3),
                                Node {
                                    flex_grow: 1.,
                                    border: UiRect::all(px(2)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                BorderColor::all(WHITE),
                                children![(Text::new("Connect"),)],
                            ));
                        });
                });
        });

    state.set(State::HandleConnectPage);
}

pub fn handle_connect_page(
    mut connect_button: Query<
        (
            Entity,
            &ConnectButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        Changed<Interaction>,
    >,
    host_field: Query<&EditableText, With<HostField>>,
    port_field: Query<&EditableText, With<PortField>>,
    player_name_field: Query<&EditableText, With<PlayerNameField>>,
    mut info_label: Query<&mut Text, With<InfoLabel>>,
    mut input_focus: ResMut<InputFocus>,
    mut server_host: ResMut<ServerHost>,
    mut server_port: ResMut<ServerPort>,
    mut player_name: ResMut<PlayerName>,
    mut state: ResMut<NextState<State>>,
    // mut r_state: ResMut<State>,
) {
    let host_field_text = host_field.iter().next().unwrap();
    let port_field_text = port_field.iter().next().unwrap();
    let player_name_text = player_name_field.iter().next().unwrap();
    for (entity, _, interaction, mut color, _, _) in &mut connect_button {
        match *interaction {
            Interaction::Pressed => {
                let host_field_string = host_field_text.value().to_string();
                let maybe_port_field_int =
                    u16::from_str(port_field_text.value().to_string().as_str());
                let player_name_field_string = player_name_text.value().to_string();
                let mut info_label = info_label.iter_mut().next().unwrap();
                if addr::parse_domain_name(host_field_string.as_str()).is_err()
                    && !host_field_string.validate_ip()
                {
                    info_label.0 = "Invalid host".to_string();
                    return;
                }
                if maybe_port_field_int.is_err() {
                    info_label.0 = "Invalid port".to_string();
                    return;
                }
                server_host.0 = host_field_string;
                server_port.0 = maybe_port_field_int.unwrap();
                player_name.0 = player_name_field_string;
                info_label.0 = "Connecting...".to_string();
                state.set(State::Connect);
                // *r_state.deref_mut() = State::Connect;
                input_focus.set(entity, FocusCause::Pressed);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                input_focus.clear();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
