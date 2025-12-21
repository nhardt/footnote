use dioxus::prelude::*;

use crate::components::Button;

#[component]
pub fn Vault() -> Element {
    rsx! {
        div { class: "flex items-center justify-center h-full",
            div { class: "max-w-md w-full p-8 bg-zinc-800 rounded-lg shadow-lg border border-zinc-700",
                h1 { class: "text-2xl font-bold text-zinc-100 mb-6 text-center", "Welcome to Footnote" }

                div { class: "flex flex-col gap-8",
                    Button{
                       onclick: move |_| {},
                       "Create Vault"
                    }

                    Button{
                       onclick: move |_| {},
                       "Open Vault"
                    }

                    Button{
                       onclick: move |_| {},
                       "Join Vault"
                    }
                }
            }
        }
    }
}
