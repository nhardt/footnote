use crate::{
    model::{
        contact::Contact,
        device::Device,
        vault::{Vault, VaultState},
    },
    util::manifest::Manifest,
};
use dioxus::prelude::*;
use std::path::PathBuf;

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

    pub fn reload_devices(&mut self) -> Result<()> {
        let vault = self.vault.read().clone();
        self.devices.set(vault.device_read()?);
        Ok(())
    }

    pub fn reload_contacts(&mut self) -> Result<()> {
        let vault = self.vault.read().clone();
        self.devices.set(vault.device_read()?);
        Ok(())
    }
}
