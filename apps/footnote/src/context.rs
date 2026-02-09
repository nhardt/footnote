use dioxus::prelude::*;

use footnote_core::model::contact::Contact;
use footnote_core::model::device::Device;
use footnote_core::model::vault::{Vault, VaultState};
use footnote_core::util::manifest::{create_manifest_local, Manifest};

use crate::route::Route;

/// AppContext is to just for grouping high level objects and easily accessing
/// them. for Footnote, this is probably:
/// Vault: mostly just a path,
/// Devices: rarely changed list of devices, needs a trigger when it changes
/// Contacts: infrequently changed list of contacts, needs a trigger
/// FileList: might be special and might not belong in the AppContext
#[derive(Clone, Copy)]
pub struct AppContext {
    pub vault: Signal<Vault>,
    pub vault_state: Signal<VaultState>,
    pub devices: Signal<Vec<Device>>,
    pub contacts: Signal<Vec<Contact>>,
    pub manifest: Signal<Manifest>,
}

impl AppContext {
    pub fn new(vault: Vault) -> Self {
        Self {
            vault: Signal::new(vault.clone()),
            vault_state: Signal::new(vault.state_read().unwrap_or(VaultState::Uninitialized)),
            devices: Signal::new(vault.device_read().expect("could not load devices")),
            contacts: Signal::new(vault.contact_read().expect("could not load contacts")),
            manifest: Signal::new(
                create_manifest_local(&vault.base_path())
                    .expect("could not load local list of files"),
            ),
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        let vault = self.vault.read().clone();
        self.vault_state
            .set(vault.state_read().unwrap_or(VaultState::Uninitialized));
        self.devices.set(vault.device_read()?);
        self.contacts.set(vault.contact_read()?);
        Ok(())
    }

    pub fn reload_manifest(&mut self) -> Result<()> {
        self.manifest.set(
            create_manifest_local(&self.vault.read().base_path())
                .expect("could not load local list of files"),
        );
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct MenuContext {
    pub menu_visible: Signal<bool>,

    pub new_note_visible: Signal<bool>,

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

    pub fn go_contacts(&mut self) {
        self.close_all();

        let nav = use_navigator();
        nav.push(Route::ContactBrowser {});
    }

    pub fn go_note(&mut self, note_path: &str) {
        self.close_all();

        let nav = use_navigator();
        tracing::info!("navigating to {}", note_path);
        nav.push(format!("/note/{}", note_path));
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

    pub fn set_new_note_visible(&mut self) {
        self.close_all();
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
