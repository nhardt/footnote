use dioxus::prelude::*;
use footnote_core::model::contact::Contact;
use footnote_core::model::device::Device;
use footnote_core::model::vault::{Vault, VaultState};
use footnote_core::util::manifest::{create_manifest_local, Manifest};

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
