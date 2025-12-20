use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
}

#[component]
pub fn FootnoteButton(
    onclick: EventHandler<MouseEvent>,
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] full_width: bool,
    /// Additional props are appended to default props
    #[props(default = String::new())]
    class: String,
    children: Element,
) -> Element {
    let base = "px-4 py-2 rounded-md font-medium focus:outline-none focus:ring-2";

    let variant_class = match variant {
        ButtonVariant::Primary => {
            "bg-indigo-600 text-white hover:bg-indigo-700 focus:ring-indigo-600"
        }
        ButtonVariant::Secondary => {
            "bg-zinc-700 text-zinc-200 border border-zinc-600 hover:bg-zinc-700"
        }
    };

    let width = if full_width { "w-full" } else { "" };
    let disabled_class = if disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    let classes = format!(
        "{} {} {} {} {}",
        base, variant_class, width, disabled_class, class
    );

    rsx! {
        button {
            class: "{classes}",
            disabled,
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}
