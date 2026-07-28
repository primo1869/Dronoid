use bevy::{
    color::palettes::{
        css::{DARK_SLATE_GRAY, RED, WHITE},
        tailwind::SLATE_300,
    },
    input_focus::{AutoFocus, FocusCause, InputFocus},
    log::LogPlugin,
    prelude::*,
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    window::{PrimaryWindow, WindowResolution},
};
use bevy_app::{App, PluginGroup, Startup, Update};
use bevy_ecs::{
    children,
    component::Component,
    entity::Entity,
    query::With,
    resource::Resource,
    schedule::{IntoScheduleConfigs, common_conditions::resource_equals},
    system::{Commands, Query, Res, ResMut},
};
use bevy_framepace::{FramepaceSettings, Limiter};
use clap::Parser;
use crossbeam_channel::Receiver;
use display_info::DisplayInfo;
use dronoid::protocol::{Kind, ServerMessage};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
    process::abort,
    str::FromStr,
};
use tokio_tungstenite::tungstenite::WebSocket;

#[derive(Resource)]
struct ServerMessageReceiver(Receiver<ServerMessage>);

impl ServerMessageReceiver {
    fn new(receiver: Receiver<ServerMessage>) -> Self {
        Self { 0: receiver }
    }
}

#[derive(Resource, Default)]
struct SpawnPoint((f32, f32));

#[derive(Resource, Default)]
struct Entities(HashMap<u32, Entity>);

#[derive(Resource, Default)]
struct GameSprites(HashMap<Kind, (f32, Handle<Image>)>);

#[derive(Resource, Default, PartialEq)]
struct State {
    pub playing: bool,
}

#[derive(Resource)]
struct ServerAddr(SocketAddr);

#[derive(Resource, Default)]
struct CurrentDisplayResolution((u32, u32));

#[derive(Component)]
struct ServerConnection(WebSocket<TcpStream>);

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = "127.0.0.1".to_string())]
    addr: String,
    #[arg(default_value_t = dronoid::defaults::PORT)]
    port: u16,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dronoid::init_logger();

    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<LogPlugin>(),
            bevy_framepace::FramepacePlugin,
        ))
        .insert_resource(Entities::default())
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .insert_resource(GameSprites::default())
        .insert_resource(ServerAddr {
            0: SocketAddr::from_str(format!("{}:{}", args.addr, args.port).as_str()).unwrap(),
        })
        .insert_resource(SpawnPoint::default())
        .insert_resource(State::default())
        .insert_resource(CurrentDisplayResolution::default())
        .add_systems(PreStartup, setup_display)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (button_system).run_if(resource_equals::<State>(State { playing: false })),
        )
        .add_systems(
            Update,
            (zoom_camera, read_server_message)
                .run_if(resource_equals::<State>(State { playing: true })),
        )
        .run();
    anyhow::Ok(())
}

fn read_server_message(
    mut entities_query: Query<&mut Transform, With<Sprite>>,
    entities: ResMut<Entities>,
    game_sprites: Res<GameSprites>,
    receiver: ResMut<ServerMessageReceiver>,
    mut commands: Commands,
) {
    loop {
        let maybe_message = receiver.0.try_recv();
        match maybe_message {
            Err(crossbeam_channel::TryRecvError::Empty) => return,
            Ok(message) => match message {
                ServerMessage::State(state) => {
                    for entity_info in state.entities_in_zone {
                        let maybe_existing_entity = entities.0.get(&entity_info.id);
                        if maybe_existing_entity.is_some() {
                            let existing_entity = maybe_existing_entity.unwrap();
                            let mut transform = entities_query.get_mut(*existing_entity).unwrap();
                            transform.translation.x = entity_info.pos.0;
                            transform.translation.y = entity_info.pos.1;
                            continue;
                        }
                        let sprite_info = game_sprites.0.get(&entity_info.kind).unwrap();
                        let mut transform =
                            Transform::from_xyz(entity_info.pos.0, entity_info.pos.1, 0.);
                        transform.scale = Vec3::new(sprite_info.0, sprite_info.0, sprite_info.0);
                        commands.spawn((Sprite::from_image(sprite_info.1.clone()), transform));
                    }
                }
                _ => {}
            },
            _ => {
                abort();
            }
        }
    }
}

fn setup_display(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut r_current_display_resolution: ResMut<CurrentDisplayResolution>,
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

fn setup(
    spawn_point: Res<SpawnPoint>,
    asset_server: Res<AssetServer>,
    mut game_sprites: ResMut<GameSprites>,
    mut framepace_settings: ResMut<FramepaceSettings>,
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

    let mut welcome_menu = commands.spawn((
        Node {
            padding: UiRect::all(Val::Percent(5.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: px(2.).all(),
            border_radius: BorderRadius::all(Val::Percent(10.)),
            ..default()
        },
        BorderColor::all(WHITE),
    ));

    let welcome_menu_id = welcome_menu
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
                    width: px(200.),
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
                EditableTextFilter::new(|c| c.is_ascii_alphabetic()),
                // TextFont::from_font_size(32.),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                BorderColor::all(SLATE_300),
                AutoFocus,
            ));
            parent.spawn((
                Button,
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
                children![(Text::new("Connect"), TextColor(Color::srgb(0.9, 0.9, 0.9)),)],
            ));
        })
        .id();

    commands
        .spawn(Node {
            width: percent(100.),
            height: percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .add_child(welcome_menu_id);
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                // input_focus.set(entity, FocusCause::Pressed);
                // **text = "Press".to_string();
                // *color = PRESSED_BUTTON.into();
                // *border_color = BorderColor::all(RED);

                // button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity, FocusCause::Pressed);
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn zoom_camera(mut query: Query<&mut Projection, With<Camera2d>>) {
    for mut projection in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 0.1;
        }
    }
}
