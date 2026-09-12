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
use bevy_color::Color;
use bevy_ecs::{children, message::MessageWriter};
use bevy_ecs::{
    entity::Entity,
    hierarchy::Children,
    query::{Changed, With},
    system::{Commands, Query, Res, ResMut},
};
use bevy_state::state::NextState;

use crate::{
    app::{ConnectButton, ConnectPage, InfoLabel, InfoMessage, PlayerName, PlayerNameField, State},
    ui::{HOVERED_BUTTON, NORMAL_BUTTON},
};

pub fn setup_connect_page(
    mut state: ResMut<NextState<State>>,
    r_player_name: Res<PlayerName>,
    mut commands: Commands,
) {
    let mut player_name_editable_text = EditableText::new(r_player_name.0.to_string().as_str());
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
            // ConnectPage,
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
                    BorderColor::all(WHITE),
                    TabGroup::new(0),
                ))
                .with_children(|parent| {
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
                                    color: Color::WHITE,
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
                                BorderColor::all(Color::WHITE),
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
    player_name_field: Query<&EditableText, With<PlayerNameField>>,
    mut info_label: MessageWriter<InfoMessage>,
    mut input_focus: ResMut<InputFocus>,
    mut player_name: ResMut<PlayerName>,
    mut state: ResMut<NextState<State>>,
) {
    let player_name_text = player_name_field.iter().next().unwrap();
    for (entity, _, interaction, mut color, _, _) in &mut connect_button {
        match *interaction {
            Interaction::Pressed => {
                let player_name_field_string = player_name_text.value().to_string();
                player_name.0 = player_name_field_string;
                info_label.write(InfoMessage("Connecting...".to_string()));
                state.set(State::Connect);
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
