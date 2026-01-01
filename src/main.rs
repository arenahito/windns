mod app;
mod components;
mod dns;
mod state;

use dioxus::prelude::*;

fn main() {
    launch(app::App);
}
