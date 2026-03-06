use dioxus::prelude::*;

pub mod file_search;
pub mod menu;
pub mod sync_service_toggle;

use footnote_core::model::vault::VaultState;

use crate::context::app_context::AppContext;

use file_search::FileSearch;
use menu::HeaderMenu;
use sync_service_toggle::SyncServiceToggle;

#[component]
pub fn Header(children: Element) -> Element {
    let app_context = use_context::<AppContext>();
    let vault_state = app_context.vault_state;

    rsx! {
        header {
            class: "sticky top-0 z-10 border-b border-zinc-800 bg-zinc-900/95 backdrop-blur-sm",
            div {
                class: "flex items-center justify-between px-4 py-3",

                HeaderMenu {}
                FileSearch {  }

                match *vault_state.read() {
                    VaultState::Primary | VaultState::SecondaryJoined => rsx! {
                        SyncServiceToggle {}
                    },
                    _ => rsx! {}
                }
            }
        }
    }
}
