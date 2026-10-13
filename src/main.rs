#[cfg(not(target_arch = "wasm32"))]
use std::os::unix::fs::FileTypeExt;

use bevy::camera::ScalingMode;
use bevy::prelude::*;

mod assets;
mod audio;
mod bombs;
mod buttons;
mod city;
mod hud;
mod rules;
mod vfx;

/// Window pixels per playfield unit. The world stays 720×1080.
#[cfg(not(target_arch = "wasm32"))]
const WINDOW_SCALE: f32 = 2.0 / 3.0;

use assets::GameAssets;
use bombs::Session;
use buttons::ColorPressed;
use rules::{VIEW_H, VIEW_W};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Playing,
    GameOver,
}

fn main() {
    // IDE terminals often omit the desktop session variables. Winit will not
    // open a window unless one of them is set, even when the socket exists.
    #[cfg(not(target_arch = "wasm32"))]
    attach_session_display();

    App::new()
        .insert_resource(ClearColor(Color::srgb(
            5.0 / 255.0,
            10.0 / 255.0,
            18.0 / 255.0,
        )))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Color Bomber".into(),
                // Same 2:3 frame as the 720×1080 playfield, drawn smaller so the
                // window is not a full screen tall. The browser canvas fills the
                // page instead, and the camera keeps the playfield aspect.
                #[cfg(not(target_arch = "wasm32"))]
                resolution: ((VIEW_W * WINDOW_SCALE) as u32, (VIEW_H * WINDOW_SCALE) as u32)
                    .into(),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_message::<ColorPressed>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Playing), bombs::begin_run)
        .add_systems(OnEnter(GameState::GameOver), bombs::end_run)
        .add_systems(
            Update,
            (
                buttons::keyboard_input,
                bombs::resolve_presses,
                ApplyDeferred,
                buttons::animate_buttons,
                bombs::spawn_bombs,
                bombs::fall_bombs,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                city::float_buildings,
                city::emit_damage_fx,
                vfx::update_particles,
                hud::update_hud,
                hud::sync_title,
                title_input,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_W,
                min_height: VIEW_H,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let assets = GameAssets::load(&asset_server);
    city::spawn_city(&mut commands, &assets);
    buttons::spawn_buttons(&mut commands, &assets);
    hud::spawn_hud(&mut commands, &assets);
    commands.insert_resource(assets);
    commands.insert_resource(Session::fresh(1));
    info!("Color Bomber ready");
}

#[cfg(not(target_arch = "wasm32"))]
fn attach_session_display() {
    fn nonempty(key: &str) -> bool {
        std::env::var_os(key).is_some_and(|value| !value.is_empty())
    }
    if nonempty("WAYLAND_DISPLAY") || nonempty("WAYLAND_SOCKET") || nonempty("DISPLAY") {
        return;
    }

    if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") {
        if let Some(name) = wayland_socket_name(std::path::Path::new(&runtime)) {
            // SAFETY: main is still the only thread. Winit reads this before it spawns workers.
            unsafe { std::env::set_var("WAYLAND_DISPLAY", &name) };
            eprintln!("Color Bomber: using Wayland display {name}");
            return;
        }
    }

    if std::path::Path::new("/tmp/.X11-unix/X0").exists() {
        unsafe { std::env::set_var("DISPLAY", ":0") };
        eprintln!("Color Bomber: using DISPLAY=:0");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn wayland_socket_name(runtime: &std::path::Path) -> Option<String> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(runtime).ok()?.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with("wayland-") || name.ends_with(".lock") {
            continue;
        }
        if entry.file_type().ok()?.is_socket() {
            names.push(name.to_string());
        }
    }
    names.sort();
    names.into_iter().next()
}

fn title_input(
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
) {
    if *state.get() == GameState::Playing {
        return;
    }
    let tap = keys.just_pressed(KeyCode::Space)
        || mouse.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();
    if tap {
        next.set(GameState::Playing);
    }
}
