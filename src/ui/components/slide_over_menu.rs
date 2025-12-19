use crate::ui::components::{icons, nav_menu_item::NavMenuItem};
use crate::ui::context::VaultContext;
use crate::ui::screens::OpenFile;
use crate::ui::Screen;
use dioxus::prelude::*;

#[component]
pub fn SlideOverMenu(
    menu_open: Signal<bool>,
    current_screen: Signal<Screen>,
    current_file: Signal<Option<OpenFile>>,
) -> Element {
    let vault_ctx = use_context::<VaultContext>();

    rsx! {
        div {
            class: "fixed inset-0 z-50 overflow-hidden",

            // Backdrop overlay
            div {
                class: "absolute inset-0 bg-zinc-950/75 transition-opacity duration-300",
                onclick: move |_| menu_open.set(false),
            }

            // Panel container
            div {
                class: "absolute inset-0 pr-10 focus:outline-none sm:pr-16",

                // Slide-over panel
                div {
                    class: "relative mr-auto h-full w-full max-w-sm transform bg-zinc-900 shadow-xl transition-transform duration-300 ease-in-out",

                    // Close button
                    div {
                        class: "absolute top-0 right-0 -mr-8 flex pt-4 pl-2 sm:-mr-10 sm:pl-4",
                        button {
                            r#type: "button",
                            onclick: move |_| menu_open.set(false),
                            class: "relative rounded-md text-zinc-400 hover:text-zinc-100 focus:outline-none focus:ring-2 focus:ring-slate-400",
                            span { class: "absolute -inset-2.5" }
                            span { class: "sr-only", "Close panel" }
                            icons::CloseIcon {}
                        }
                    }

                    // Panel content
                    div {
                        class: "relative h-full overflow-y-auto p-6 flex flex-col",

                        // Navigation menu
                        nav { class: "flex flex-col gap-2",
                            NavMenuItem {
                                screen: Screen::Editor,
                                current_screen,
                                menu_open,
                                label: "Editor".to_string(),
                                icons::EditIcon {}
                            }

                            NavMenuItem {
                                screen: Screen::Contacts,
                                current_screen,
                                menu_open,
                                label: "Contacts".to_string(),
                                icons::ContactsIcon {}
                            }

                            NavMenuItem {
                                screen: Screen::Sync,
                                current_screen,
                                menu_open,
                                label: "Sync".to_string(),
                                icons::SyncIcon {}
                            }
                        }

                        // Spacer to push Switch Vault to bottom
                        div { class: "flex-1" }

                        // Switch Vault button at bottom
                        div { class: "border-t border-zinc-700 pt-4 mt-4",
                            button {
                                onclick: move |_| {
                                    let mut vault_ctx = vault_ctx.clone();
                                    let mut current_file = current_file.clone();
                                    vault_ctx.clear_vault();
                                    current_file.set(None);
                                    menu_open.set(false);

                                    // Clear config when switching vaults
                                    spawn(async move {
                                        if let Err(e) = crate::ui::config::AppConfig::delete() {
                                            tracing::warn!("Failed to delete config: {}", e);
                                        }
                                    });
                                },
                                class: "flex w-full items-center gap-x-3 rounded-md p-3 text-sm font-semibold text-zinc-200 hover:bg-zinc-800",
                                icons::SwitchIcon {}
                                "Switch Vault"
                            }
                        }
                    }
                }
            }
        }
    }
}
