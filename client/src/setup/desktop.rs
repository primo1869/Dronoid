use bevy::window::{PrimaryWindow, Window, WindowResolution};
use bevy_ecs::{query::With, system::Query};
use display_info::DisplayInfo;

// use crate::CurrentDisplayResolution;

pub fn setup_display(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    // mut r_current_display_resolution: ResMut<CurrentDisplayResolution>,
) {
    let mut current_display_resolution = (1280, 720);
    let display_infos = DisplayInfo::all().unwrap();

    for display_info in display_infos {
        if display_info.is_builtin {
            current_display_resolution.0 = display_info.width;
            current_display_resolution.1 = display_info.height;
            break;
        }
    }

    for mut window in window.iter_mut() {
        window.borderless_game = false;
        window.fullsize_content_view = false;
        window.resizable = true;
        window.resolution = WindowResolution::new(
            (current_display_resolution.0 as f64 * (2. / 3.)) as u32,
            (current_display_resolution.1 as f64 * (2. / 3.)) as u32,
        )
    }
    // r_current_display_resolution.0 = current_display_resolution;
}
