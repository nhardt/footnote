#![cfg(feature = "ui")]

mod app;

pub fn launch() {
    dioxus::launch(app::App);
}
