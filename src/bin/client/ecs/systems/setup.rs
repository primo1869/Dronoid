use std::ops::DerefMut;

use bevy::{
    color::palettes::{
        css::{DARK_SLATE_GRAY, WHITE},
        tailwind::SLATE_300,
    },
    input_focus::AutoFocus,
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
    // fixme : startup
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
        // whole window rectangle
        .spawn(Node {
            width: percent(100.),
            height: percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // central rectangle
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Percent(5.)),
                        // justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        align_content: AlignContent::Center,
                        border: px(2.).all(),
                        border_radius: BorderRadius::all(Val::Percent(10.)),
                        ..default()
                    },
                    BorderColor::all(WHITE),
                ))
                .with_children(|parent| {
                    // Host field
                    parent
                        .spawn(Node {
                            padding: UiRect::all(Val::Percent(5.)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            // border: px(2.).all(),
                            // border_radius: BorderRadius::all(Val::Percent(10.)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    margin: UiRect::right(Val::Px(10.0)),
                                    ..Default::default()
                                },
                                Text::new("Host"),
                            ));
                            parent.spawn((
                                Node {
                                    width: px(400.),
                                    height: px(50.),
                                    border: px(2.).all(),
                                    padding: px(8.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                EditableText {
                                    max_characters: Some(8),
                                    ..default()
                                },
                                TextCursorStyle::default(),
                                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic()),
                                // TextFont::from_font_size(32.),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                                AutoFocus,
                            ));
                        });
                    //
                    parent
                        .spawn(Node {
                            padding: UiRect::all(Val::Percent(5.)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            // border: px(2.).all(),
                            // border_radius: BorderRadius::all(Val::Percent(10.)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    margin: UiRect::right(Val::Px(10.0)),
                                    ..Default::default()
                                },
                                Text::new("Nickname"),
                            ));
                            parent.spawn((
                                Node {
                                    // width: px(400.),
                                    // height: px(50.),
                                    border: px(2.).all(),
                                    padding: px(8.).all(),
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    ..default()
                                },
                                EditableText {
                                    max_characters: Some(8),
                                    visible_width: Some(30.),
                                    ..default()
                                },
                                TextCursorStyle::default(),
                                EditableTextFilter::new(|c| c.is_ascii_alphabetic()),
                                // TextFont::from_font_size(32.),
                                BackgroundColor(DARK_SLATE_GRAY.into()),
                                BorderColor::all(SLATE_300),
                            ));
                            parent.spawn((
                                components::ConnectButton,
                                Node {
                                    width: px(200),
                                    height: px(50),
                                    border: UiRect::all(px(5)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Percent(10.)),
                                    margin: UiRect::left(Val::Px(10.)),
                                    ..default()
                                },
                                BorderColor::all(Color::WHITE),
                                BackgroundColor(Color::BLACK),
                                children![(
                                    Text::new("Connect"),
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                )],
                            ));
                        });
                });
        });

    *state.deref_mut() = resources::State::ShowConnectPage;
}
