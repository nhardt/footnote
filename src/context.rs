use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct VaultContext {
    vault_path: Signal<Option<PathBuf>>,
}

impl VaultContext {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            vault_path: Signal::new(path),
        }
    }

    pub fn set_vault(&mut self, path: PathBuf) {
        self.vault_path.set(Some(path));
    }

    pub fn get_vault(&self) -> Option<PathBuf> {
        self.vault_path.cloned()
    }

    pub fn _clear_vault(&mut self) {
        self.vault_path.set(None);
    }
}
