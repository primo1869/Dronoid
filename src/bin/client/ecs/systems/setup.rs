use std::ops::DerefMut;

use bevy::{
    color::palettes::{
        css::{DARK_SLATE_GRAY, RED, WHITE},
        tailwind::SLATE_300,
    },
    input_focus::{self, AutoFocus},
    prelude::*,
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    window::{PrimaryWindow, WindowResolution},
};
use bevy_ecs::{
    children,
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use bevy_framepace::{FramepaceSettings, Limiter};
use display_info::DisplayInfo;

use crate::ecs::{components, resources};

pub fn setup_display(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut r_current_display_resolution: ResMut<resources::CurrentDisplayResolution>,
) {
    let mut current_display_resolution = (1280, 720);
    if let Ok(displays) = DisplayInfo::all() {
        for display in displays {
            if display.is_builtin {
                current_display_resolution.0 = display.width;
                current_display_resolution.1 = display.height;
            }
        }
    }
    for mut window in q_window.iter_mut() {
        window.borderless_game = false;
        window.fullsize_content_view = false;
        window.resizable = false;
        window.resolution = WindowResolution::new(
            current_display_resolution.0 / 2,
            current_display_resolution.1 / 2,
        )
    }
    r_current_display_resolution.0 = current_display_resolution;
}

pub fn setup_connect_page(
    spawn_point: Res<resources::SpawnPoint>,
    asset_server: Res<AssetServer>,
    mut game_sprites: ResMut<resources::GameSprites>,
    mut framepace_settings: ResMut<FramepaceSettings>,
    mut state: ResMut<resources::State>,
    mut commands: Commands,
) {
    framepace_settings.limiter = Limiter::from_framerate(60.0);
    game_sprites.0.insert(
        dronoid::protocol::Kind::Mineral,
        (1. / 128., asset_server.load("textures/mineral.png")),
    );
    game_sprites.0.insert(
        dronoid::protocol::Kind::Dronoid,
        (3. / 128., asset_server.load("textures/dronoid.png")),
    );
    game_sprites.0.insert(
        dronoid::protocol::Kind::Factory,
        (6. / 128., asset_server.load("textures/factory.png")),
    );
    game_sprites.0.insert(
        dronoid::protocol::Kind::Spawn,
        (9. / 128., asset_server.load("textures/spawn.png")),
    );

    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(spawn_point.0.0, spawn_point.0.1, 0.)),
    ));

    commands
        .spawn(Node {
            width: percent(100.),
            height: percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
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
                                Text::new("Host"),
                            ));
                            parent.spawn((
                                Node {
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                EditableText {
                                    cursor_width: 0.5,
                                    visible_width: Some(30.),
                                    max_characters: Some(62),
                                    ..default()
                                },
                                TextCursorStyle {
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic()),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                                AutoFocus,
                            ));
                            parent.spawn((
                                Node {
                                    ..Default::default()
                                },
                                Text::new("Port"),
                            ));
                            parent.spawn((
                                Node {
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                EditableText {
                                    cursor_width: 0.5,
                                    visible_width: Some(5.),
                                    max_characters: Some(5),
                                    ..default()
                                },
                                TextCursorStyle::default(),
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
                                Text::new("Nickname"),
                            ));
                            parent.spawn((
                                Node {
                                    border: px(2.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                EditableText {
                                    cursor_width: 0.5,
                                    max_characters: Some(15),
                                    visible_width: Some(15.),
                                    ..default()
                                },
                                TextCursorStyle::default(),
                                EditableTextFilter::new(|c| c.is_ascii_alphabetic()),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                            ));
                            parent.spawn((
                                components::ConnectButton,
                                Button,
                                Interaction::default(),
                                Node {
                                    flex_grow: 1.,
                                    border: UiRect::all(px(2)),
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                BorderColor::all(Color::WHITE),
                                // BackgroundColor(Color::BLACK),
                                children![(
                                    Text::new("Connect"),
                                    // TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                )],
                            ));
                        });
                });
        });

    *state.deref_mut() = resources::State::ShowConnectPage;
}
