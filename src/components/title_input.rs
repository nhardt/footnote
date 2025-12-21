use dioxus::prelude::*;

#[component]
pub fn TitleInput(value: String, on_change: EventHandler<String>) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut edit_value = use_signal(|| value.clone());

    let value_for_effect = value.clone();
    use_effect(move || {
        edit_value.set(value_for_effect.clone());
    });

    let value_for_blur = value.clone();
    let value_for_keydown = value.clone();
    let value_for_escape = value.clone();
    let value_for_click = value.clone();

    rsx! {
        if is_editing() {
            input {
                r#type: "text",
                class: "editable-title editing",
                value: "{edit_value}",
                oninput: move |evt| edit_value.set(evt.value()),
                onblur: move |_| {
                    let new_value = edit_value().trim().to_string();
                    let is_valid = !new_value.is_empty()
                        && new_value.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '.' || c == '-' || c == '_');

                    if is_valid && new_value != value_for_blur {
                        on_change.call(new_value);
                    }
                    is_editing.set(false);
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        evt.prevent_default();
                        let new_value = edit_value().trim().to_string();
                        let is_valid = !new_value.is_empty()
                            && new_value.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '.' || c == '-' || c == '_');

                        if is_valid && new_value != value_for_keydown {
                            on_change.call(new_value);
                        }
                        is_editing.set(false);
                    } else if evt.key() == Key::Escape {
                        evt.prevent_default();
                        edit_value.set(value_for_escape.clone());
                        is_editing.set(false);
                    }
                },
                autofocus: true,
            }
        } else {
            div {
                class: "editable-title",
                onclick: move |_| {
                    edit_value.set(value_for_click.clone());
                    is_editing.set(true);
                },
                "{value}"
            }
        }
    }
}
