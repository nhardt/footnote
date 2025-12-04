mod app;
mod markdown;

pub fn launch() {
    dioxus::launch(app::App);
}
