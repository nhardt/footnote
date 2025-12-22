use dioxus::prelude::*;

#[component]
pub fn Contacts() -> Element {
    //vault::contacts_read
    rsx! {
        div {
            div { class: "mb-8",
                NewContactForm {
                    new_contact_state,
                    reload_trigger,
                }
            }

            div {
                if !trusted_contacts().is_empty() {
                    div { class: "space-y-2",
                        for (petname, contact) in trusted_contacts().iter() {
                            Contact {
                                key: "{petname}",
                                petname: petname.clone(),
                                username: contact.username.clone(),
                                device_count: contact.devices.len(),
                            }
                        }
                    }
                }
            }
        }
    }
}
