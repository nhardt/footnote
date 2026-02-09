use dioxus::prelude::*;

use crate::context::AppContext;
use crate::context::MenuContext;

#[component]
pub fn ImportContactModal() -> Element {
    let mut contact_json = use_signal(|| String::new());
    let mut nickname = use_signal(|| String::new());
    let mut err_message = use_signal(|| String::new());
    let mut app_context = use_context::<AppContext>();

    use_effect(move || {
        let imported_contact_data = consume_context::<MenuContext>()
            .imported_contact_string
            .read()
            .clone();

        if !imported_contact_data.is_empty() {
            contact_json.set(imported_contact_data.clone());
        }
    });

    let import_contact = move |_| {
        let vault = app_context.vault.read().clone();
        match vault.contact_import(&nickname.read().clone(), &contact_json.read().clone()) {
            Ok(()) => {
                app_context
                    .contacts
                    .set(vault.contact_read().expect("could not load contacts"));
                consume_context::<MenuContext>().close_all();
            }
            Err(e) => err_message.set(format!("Failed to import contact: {e}")),
        };
    };

    rsx! {
        div {
            id: "import-modal",
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full h-[90vh] flex flex-col",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg text-zinc-300 ont-semibold font-mono",
                        "Import Contact"
                    }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "Add someone to your trust network"
                    }
                }
                div { class: "p-6 flex-1 min-h-0 flex flex-col gap-4",
                    div {
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Nickname"
                            span { class: "text-zinc-500 font-normal ml-1",
                                "(how you'll reference them when sharing)"
                            }
                        }
                        input {
                            class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            placeholder: "alice",
                            r#type: "text",
                            value: "{nickname}",
                            oninput: move |e| nickname.set(e.value())
                        }
                    }
                    div { class: "flex-3 min-h-0 flex flex-col",
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Contact Record"
                        }
                        textarea {
                            class: "flex-1 w-full px-4 py-3 bg-zinc-950 border border-zinc-700 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            value: "{contact_json}",
                            oninput: move |e| contact_json.set(e.value())
                        }
                    }
                    div {
                        class: "text-sm text-red-400 font-mono",
                        style: "display: none",
                        "{err_message}"
                    }
                    div { class: "flex gap-3",
                        button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                            onclick: move |_| consume_context::<MenuContext>().close_all(),
                            "Cancel"
                        }
                        button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                            onclick: import_contact,
                            "Import"
                        }
                    }
                }
            }
        }
    }
}
