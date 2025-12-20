use crate::core::user::import_from_string;
use crate::ui::context::VaultContext;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum NewContactState {
    Idle,
    Form,
    Success { petname: String },
    Error(String),
}

#[component]
pub fn NewContactForm(
    new_contact_state: Signal<NewContactState>,
    reload_trigger: Signal<i32>,
) -> Element {
    let mut json_input = use_signal(|| String::new());
    let mut petname_input = use_signal(|| String::new());
    let vault_ctx = use_context::<VaultContext>();

    rsx! {
        div {
            // Add Contact button
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-bold text-zinc-100", "Contacts" }
                if matches!(new_contact_state(), NewContactState::Idle) {
                    button {
                        class: "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700",
                        onclick: move |_| {
                            new_contact_state.set(NewContactState::Form);
                        },
                        "Add Contact"
                    }
                }
            }

            // Contact form UI
            match new_contact_state() {
                NewContactState::Form => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-zinc-700 rounded-md",
                        div { class: "font-semibold text-zinc-100 mb-4", "Add New Contact" }

                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-zinc-300 mb-2",
                                "Petname (nickname for this contact)"
                            }
                            input {
                                class: "w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                r#type: "text",
                                placeholder: "e.g., alice",
                                value: "{petname_input}",
                                oninput: move |e| petname_input.set(e.value().clone()),
                            }
                        }

                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-zinc-300 mb-2",
                                "Contact JSON"
                            }
                            textarea {
                                class: "w-full px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 font-mono text-xs focus:outline-none focus:ring-2 focus:ring-indigo-500",
                                rows: "10",
                                placeholder: "Paste the JSON contact record here...",
                                value: "{json_input}",
                                oninput: move |e| json_input.set(e.value().clone()),
                            }
                        }

                        div { class: "flex gap-2 justify-end",
                            button {
                                class: "px-4 py-2 bg-zinc-700 text-zinc-100 rounded-md hover:bg-zinc-600",
                                onclick: move |_| {
                                    new_contact_state.set(NewContactState::Idle);
                                    json_input.set(String::new());
                                    petname_input.set(String::new());
                                },
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700",
                                onclick: move |_| {
                                    let mut new_contact_state = new_contact_state.clone();
                                    let mut reload_trigger = reload_trigger.clone();
                                    let vault_ctx = vault_ctx.clone();
                                    let json_str = json_input().clone();
                                    let petname = petname_input().clone();
                                    let mut json_input = json_input.clone();
                                    let mut petname_input = petname_input.clone();

                                    spawn(async move {
                                        let vault_path = match vault_ctx.get_vault() {
                                            Some(path) => path,
                                            None => {
                                                new_contact_state.set(NewContactState::Error(
                                                    "No vault path available".to_string()
                                                ));
                                                return;
                                            }
                                        };

                                        if petname.trim().is_empty() {
                                            new_contact_state.set(NewContactState::Error(
                                                "Petname cannot be empty".to_string()
                                            ));
                                            return;
                                        }

                                        if json_str.trim().is_empty() {
                                            new_contact_state.set(NewContactState::Error(
                                                "Contact JSON cannot be empty".to_string()
                                            ));
                                            return;
                                        }

                                        match import_from_string(&vault_path, &json_str, &petname).await {
                                            Ok(_) => {
                                                new_contact_state.set(NewContactState::Success {
                                                    petname: petname.clone()
                                                });
                                                reload_trigger.set(reload_trigger() + 1);
                                                json_input.set(String::new());
                                                petname_input.set(String::new());
                                            }
                                            Err(e) => {
                                                new_contact_state.set(NewContactState::Error(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                "Import Contact"
                            }
                        }
                    }
                },
                NewContactState::Success { ref petname } => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-green-600 rounded-md",
                        div { class: "font-semibold text-zinc-100", "✓ Contact '{petname}' added successfully!" }
                        button {
                            class: "mt-2 text-sm text-indigo-400 hover:underline",
                            onclick: move |_| new_contact_state.set(NewContactState::Idle),
                            "Done"
                        }
                    }
                },
                NewContactState::Error(ref error) => rsx! {
                    div { class: "mt-4 p-4 bg-zinc-800 border border-red-600 rounded-md",
                        div { class: "font-semibold text-red-400", "✗ Error" }
                        div { class: "text-sm text-zinc-300 mt-1", "{error}" }
                        button {
                            class: "mt-2 text-sm text-indigo-400 hover:underline",
                            onclick: move |_| new_contact_state.set(NewContactState::Form),
                            "Try Again"
                        }
                    }
                },
                NewContactState::Idle => rsx! {},
            }
        }
    }
}
