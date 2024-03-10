// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy::window::{CompositeAlphaMode, PrimaryWindow, WindowMode};
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use std::io::Cursor;
use winit::window::Icon;

mod pet;
mod state;

use pet::Pet;
use state::State;

fn main() {
    env_logger::init();
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::rgba(0.5, 0.5, 0.5, 0.)))
        .insert_resource(CursorWorldPos(None))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy game".to_string(), // ToDo
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,

                transparent: true,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,

                window_level: bevy::window::WindowLevel::AlwaysOnTop,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(pet::PetPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, set_window_icon)
        .add_systems(Update, get_world_pos)
        .add_systems(Update, update_cursor_hit_test)
        .run();
}

#[derive(Resource)]
struct CursorWorldPos(Option<Vec2>);

fn get_world_pos(
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    mut q_primary_window: Query<&mut Window, With<PrimaryWindow>>,

    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let mut primary_window = q_primary_window.single_mut();
    let (main_camera, main_camera_transform) = q_camera.single();

    cursor_world_pos.0 = primary_window
        .cursor_position()
        .and_then(|cursor_pos| main_camera.viewport_to_world_2d(main_camera_transform, cursor_pos));
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::BLUE),
            ..Default::default()
        })
        .insert(Pet::new(true));

    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::BLUE),
            ..Default::default()
        })
        .insert(Pet::new(false));
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

fn update_cursor_hit_test(
    cursor_world_pos: Res<CursorWorldPos>,
    mut q_primary_window: Query<&mut Window, With<PrimaryWindow>>,
    q_pet: Query<(&Pet, &Transform), With<Pet>>,
) {
    let mut primary_window = q_primary_window.single_mut();

    // If the window has decorations (e.g. a border) then it should be clickable
    primary_window.cursor.hit_test = true;
    if cursor_world_pos.0.is_none() {
        return;
    }

    // If the cursor is within the radius of the Bevy logo make the window clickable otherwise the window is not clickable
    let (pet, pet_transform) = q_pet.single();
    // primary_window.cursor.hit_test = pet_transform
    //     .translation
    //     .truncate()
    //     .distance_squared(cursor_world_pos)
    //     < pet.size.y;
    let cursor = (cursor_world_pos.0.unwrap()
        + (Vec2::new(primary_window.width(), -primary_window.height())) / 2.)
        * Vec2::new(1., -1.);
    primary_window.cursor.hit_test = cursor.x >= pet.pos.x
        && cursor.x <= pet.pos.x + pet.size.x
        && cursor.y >= pet.pos.y
        && cursor.y <= pet.pos.y + pet.size.y;
}
