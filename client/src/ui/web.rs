pub fn setup_connect_page(
    mut r_state: ResMut<resources::State>,

    r_player_name: Res<resources::PlayerName>,
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
            components::ConnectPage,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(20.),
                    ..default()
                },
                components::InfoLabel,
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
            components::ConnectPage,
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
                                components::PlayerNameField,
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
                                components::ConnectButton,
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

    *r_state.deref_mut() = resources::State::HandleConnectPage;
}

pub fn handle_connect_page(
    mut q_connect_button: Query<
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
    q_player_name_field: Query<&EditableText, With<components::PlayerNameField>>,
    mut q_info_label: Query<&mut Text, With<components::InfoLabel>>,
    mut r_input_focus: ResMut<InputFocus>,
    mut r_player_name: ResMut<resources::PlayerName>,
    mut r_state: ResMut<resources::State>,
) {
    let player_name_text = q_player_name_field.iter().next().unwrap();
    for (entity, _, interaction, mut color, _, _) in &mut q_connect_button {
        match *interaction {
            Interaction::Pressed => {
                let player_name_field_string = player_name_text.value().to_string();
                let mut info_label = q_info_label.iter_mut().next().unwrap();
                r_player_name.0 = player_name_field_string;
                info_label.0 = "Connecting...".to_string();
                *r_state.deref_mut() = resources::State::Connect;
                r_input_focus.set(entity, FocusCause::Pressed);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                r_input_focus.clear();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
