use crate::model::vault::Vault;
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct VaultContext {
    vault: Signal<Vault>,
}

impl VaultContext {
    pub fn new(vault: Vault) -> Self {
        Self {
            vault: Signal::new(vault),
        }
    }

    pub fn get(&self) -> Vault {
        self.vault.cloned()
    }

    pub fn set(&mut self, vault: Vault) {
        self.vault.set(vault);
    }
}
