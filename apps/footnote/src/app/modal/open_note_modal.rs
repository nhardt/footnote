use dioxus::prelude::*;

use footnote_core::util::tree_node::build_tree_from_manifest;
use footnote_core::util::tree_node::TreeNode;

use crate::context::AppContext;

#[component]
pub fn NoteSelectModal(onselect: EventHandler, oncancel: EventHandler) -> Element {
    let app_context = use_context::<AppContext>();
    let tree = use_memo(move || build_tree_from_manifest(&app_context.manifest.read()));

    let mut root_children: Vec<_> = tree().children.into_iter().collect();
    root_children.sort_by(|(_, a), (_, b)| {
        let a_is_folder = !a.children.is_empty();
        let b_is_folder = !b.children.is_empty();
        match (a_is_folder, b_is_folder) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| oncancel.call(()),

            div {
                class: "w-full max-w-2xl h-[80vh] border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl flex flex-col",
                onclick: move |evt| evt.stop_propagation(),

                div {
                    class: "sticky top-0 py-3 px-4 border-b bg-zinc-900 border-zinc-800 flex justify-between items-center",
                    h3 { class: "text-sm font-semibold", "Browse Files" }
                    button {
                        class: "p-1 rounded transition-colors text-zinc-500 hover:text-zinc-300",
                        onclick: move |_| oncancel.call(()),
                        "✕"
                    }
                }

                div {
                    class: "flex-1 overflow-y-auto p-2",
                    for (name, child) in root_children {
                        TreeNodeView {
                            name: name,
                            node: child,
                            onselect: onselect,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TreeNodeView(name: String, node: TreeNode, onselect: EventHandler) -> Element {
    let is_folder = !node.children.is_empty();

    if is_folder {
        let mut sorted_children: Vec<_> = node.children.values().cloned().collect();
        sorted_children.sort_by(|a, b| {
            let a_is_folder = !a.children.is_empty();
            let b_is_folder = !b.children.is_empty();
            match (a_is_folder, b_is_folder) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        rsx! {
            BrowserRowFolder {
                name: node.name.clone(),
                open: false,
                for child in sorted_children {
                    TreeNodeView {
                        name: child.name.clone(),
                        node: child,
                        onselect: onselect
                    }
                }
            }
        }
    } else {
        rsx! {
            BrowserRowFile {
                node: node,
                onselect: onselect
            }
        }
    }
}

#[component]
fn BrowserRowFolder(name: String, open: bool, children: Element) -> Element {
    let mut open_signal = use_signal(|| open);
    let toggle_open = move |_| open_signal.set(!open_signal());

    rsx! {
        if open_signal() {
            button {
                class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick: toggle_open,
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "M19 9l-7 7-7-7",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                    }
                }
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path { d: "M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" }
                }
                span { class: "font-medium", "{name}" }
            }
            div { class: "ml-6", {children} }
        } else {
            button {
                class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick: toggle_open,
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "M9 5l7 7-7 7",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                    }
                }
                svg {
                    class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path { d: "M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" }
                }
                span { class: "font-medium", "{name}" }
            }
        }
    }
}

#[component]
fn BrowserRowFile(node: TreeNode, onselect: EventHandler) -> Element {
    let nav = use_navigator();
    let app_context = use_context::<AppContext>();
    let path_clone = node.full_path.clone();

    let onclick = move |_| {
        if let Some(relative_path) = &path_clone {
            nav.push(format!(
                "/notes/{}",
                app_context
                    .vault
                    .read()
                    .base_path()
                    .join(relative_path)
                    .to_string_lossy()
                    .to_string()
            ));
            onselect(());
        }
    };

    rsx! {
        button {
            class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
            onclick: onclick,
            div { class: "flex-shrink-0 w-4 h-4" }
            svg {
                class: "flex-shrink-0 w-4 h-4 text-zinc-500",
                fill: "currentColor",
                view_box: "0 0 20 20",
                path {
                    clip_rule: "evenodd",
                    d: "M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z",
                    fill_rule: "evenodd",
                }
            }
            span { class: "text-zinc-300", "{node.name}" }
        }
    }
}
