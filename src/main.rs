mod app;
mod components;
mod dns;
mod state;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new().with_menu(None).with_window(
                WindowBuilder::new()
                    .with_title("Windows DNS Switcher")
                    .with_inner_size(LogicalSize::new(850.0, 820.0)),
            ),
        )
        .launch(app::App);
}
