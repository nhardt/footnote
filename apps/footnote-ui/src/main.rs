use dioxus::prelude::*;
use footnote_lib::App;
use tracing::Level;

fn main() {
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    tracing::trace!("trace");
    tracing::debug!("debug");
    tracing::info!("info");
    tracing::warn!("warn");
    tracing::error!("error");
    dioxus::launch(App);
}
