mod app;
mod components;
mod dns;
mod state;

use dioxus::desktop::tao::dpi::{LogicalSize, PhysicalPosition};
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use dns::{WindowState, load_config, validate_window_state};

fn load_icon() -> Option<Icon> {
    let icon_bytes = include_bytes!("../icons/icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).ok()
}

fn main() {
    let config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config, using defaults: {}", e);
            Default::default()
        }
    };
    let saved_state = config.window.clone().unwrap_or_default();

    let initial_width = saved_state.width.max(WindowState::MIN_WIDTH);
    let initial_height = saved_state.height.max(WindowState::MIN_HEIGHT);

    let window_builder = WindowBuilder::new()
        .with_title("Windows DNS Switcher")
        .with_window_icon(load_icon())
        .with_inner_size(LogicalSize::new(
            initial_width as f64,
            initial_height as f64,
        ))
        .with_position(PhysicalPosition::new(saved_state.x, saved_state.y));

    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_menu(None)
                .with_window(window_builder)
                .with_on_window({
                    let saved_state = saved_state.clone();
                    move |window, _| {
                        let monitors: Vec<_> = window.available_monitors().collect();
                        let primary = window.primary_monitor();
                        let validated =
                            validate_window_state(&saved_state, &monitors, primary.as_ref());

                        if validated.x != saved_state.x || validated.y != saved_state.y {
                            window.set_outer_position(PhysicalPosition::new(
                                validated.x,
                                validated.y,
                            ));
                        }

                        if validated.width != saved_state.width
                            || validated.height != saved_state.height
                        {
                            window.set_inner_size(LogicalSize::new(
                                validated.width as f64,
                                validated.height as f64,
                            ));
                        }

                        if validated.maximized {
                            window.set_maximized(true);
                        }
                    }
                }),
        )
        .launch(app::App);
}
