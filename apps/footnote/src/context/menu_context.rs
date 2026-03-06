use dioxus::prelude::*;

use crate::route::Route;

#[derive(Clone, Copy)]
pub struct MenuContext {
    pub menu_visible: Signal<bool>,

    pub new_note_visible: Signal<bool>,
    pub new_note_path_prefix: Signal<Option<String>>,

    pub note_browser_visible: Signal<bool>,

    pub imported_contact_string: Signal<String>,
    pub import_contact_visible: Signal<bool>,

    pub share_contact_visible: Signal<bool>,

    pub listening_device_url: Signal<String>,
    pub pair_with_listener_visible: Signal<bool>,

    pub listen_for_pair_visible: Signal<bool>,
}

impl MenuContext {
    pub fn new() -> Self {
        Self {
            menu_visible: Signal::new(false),

            new_note_visible: Signal::new(false),
            new_note_path_prefix: Signal::new(Option::<String>::None),

            note_browser_visible: Signal::new(false),

            share_contact_visible: Signal::new(false),

            imported_contact_string: Signal::new(String::new()),
            import_contact_visible: Signal::new(false),

            listen_for_pair_visible: Signal::new(false),

            listening_device_url: Signal::new(String::new()),
            pair_with_listener_visible: Signal::new(false),
        }
    }

    pub fn toggle_menu(&mut self) {
        let vis = *self.menu_visible.read();
        self.menu_visible.set(!vis);
    }

    pub fn go_home(&mut self) {
        self.close_all();

        let nav = use_navigator();
        nav.push(Route::Home {});
    }

    pub fn go_profile(&mut self) {
        self.close_all();

        let nav = use_navigator();
        nav.push(Route::Profile {});
    }

    pub fn go_forward(&mut self) {
        self.close_all();

        let nav = use_navigator();
        nav.go_forward();
    }

    pub fn go_back(&mut self) {
        self.close_all();

        let nav = use_navigator();
        nav.go_back();
    }

    pub fn go_contacts(&mut self) {
        self.close_all();

        let nav = use_navigator();
        nav.push(Route::ContactBrowser {});
    }

    pub fn go_note(&mut self, vault_relative_path: &str) {
        self.close_all();

        let nav = use_navigator();
        tracing::info!("navigating to {}", vault_relative_path);
        nav.push(format!("/note/{}", vault_relative_path));
    }

    pub fn close_all(&mut self) {
        self.menu_visible.set(false);
        self.new_note_visible.set(false);
        self.note_browser_visible.set(false);
        self.share_contact_visible.set(false);
        self.import_contact_visible.set(false);
        self.listen_for_pair_visible.set(false);
        self.pair_with_listener_visible.set(false);
    }

    pub fn set_new_note_visible(&mut self, path_prefix: Option<String>) {
        self.close_all();
        self.new_note_path_prefix.set(path_prefix);
        self.new_note_visible.set(true);
    }

    pub fn set_note_browser_visible(&mut self) {
        self.close_all();
        self.note_browser_visible.set(true);
    }

    pub fn set_share_contact_visible(&mut self) {
        self.close_all();
        self.share_contact_visible.set(true);
    }

    pub fn set_import_contact_visible(&mut self, imported_contact: &str) {
        self.close_all();
        self.imported_contact_string
            .set(imported_contact.to_string());
        self.import_contact_visible.set(true);
    }

    pub fn set_listen_for_pair_visible(&mut self) {
        self.close_all();
        self.listen_for_pair_visible.set(true);
    }

    pub fn set_pair_with_listener_visible(&mut self, listener_url: &str) {
        self.close_all();
        self.listening_device_url.set(listener_url.to_string());
        self.pair_with_listener_visible.set(true);
    }
}
