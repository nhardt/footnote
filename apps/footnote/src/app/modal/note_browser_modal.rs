use dioxus::prelude::*;
use dx_icons::lucide::*;

use footnote_core::util::tree_node::build_tree_from_manifest;
use footnote_core::util::tree_node::TreeNode;

use crate::context::AppContext;
use crate::context::MenuContext;

#[component]
pub fn NoteBrowserModal() -> Element {
    let app_context = use_context::<AppContext>();
    use_effect(|| {
        if let Err(e) = consume_context::<AppContext>().reload_manifest() {
            tracing::warn!("could not reload manifest {}", e)
        }
    });
    let tree = use_memo(move || build_tree_from_manifest(&app_context.manifest.read()));

    let mut root_children: Vec<_> = tree().children.into_iter().collect();
    root_children.sort_by(|(_, a), (_, b)| {
        if a.name == "footnotes" {
            return std::cmp::Ordering::Less;
        }
        if b.name == "footnotes" {
            return std::cmp::Ordering::Greater;
        }
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
            class: "fixed text-zinc-100 inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            onclick: move |_| consume_context::<MenuContext>().close_all(),

            div {
                class: "w-full max-w-2xl h-[80vh] border border-zinc-700 rounded-lg bg-zinc-900 shadow-2xl flex flex-col",
                onclick: move |evt| evt.stop_propagation(),

                div {
                    class: "sticky top-0 py-3 px-4 border-b bg-zinc-900 border-zinc-800 flex justify-between items-center",
                    h3 { class: "text-sm font-semibold", "Browse Files" }
                    button {
                        class: "p-1 rounded transition-colors text-zinc-500 hover:text-zinc-300",
                        onclick: move |_| consume_context::<MenuContext>().close_all(),
                        "✕"
                    }
                }

                div {
                    class: "flex-1 overflow-y-auto p-2",
                    for (name, child) in root_children {
                        TreeNodeView {
                            name: &name,
                            node: child,
                            is_footnote: name == "footnotes",
                            path_prefix: name.clone()
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TreeNodeView(name: String, node: TreeNode, is_footnote: bool, path_prefix: String) -> Element {
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

        let child_path_prefix = format!("{}/{}", path_prefix, name);
        rsx! {
            BrowserRowFolder {
                name: node.name.clone(),
                open: false,
                is_footnote: is_footnote,
                path_prefix: path_prefix,
                for child in sorted_children {
                    TreeNodeView {
                        name: child.name.clone(),
                        node: child,
                        is_footnote: is_footnote,
                        path_prefix: child_path_prefix.clone()
                    }
                }
            }
        }
    } else {
        rsx! {
            BrowserRowFile {
                node: node,
                is_footnote: is_footnote
            }
        }
    }
}

#[component]
fn BrowserRowFolder(
    name: String,
    open: bool,
    children: Element,
    is_footnote: bool,
    path_prefix: String,
) -> Element {
    let mut open_signal = use_signal(|| open);
    let toggle_open = move |_| open_signal.set(!open_signal());

    rsx! {
        if open_signal() {
            button {
                class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick: toggle_open,
                Icon { icon: LucideIcon::FolderOpen, size: 18 }
                span { class: "font-medium",
                    class: if is_footnote { "text-amber-400" } else { "text-zinc-200" },
                    "{name}"
                }
                if !is_footnote {
                    div { class: "flex-1" }
                    button {
                        onclick: {
                            let prefix = path_prefix.clone();
                            move |evt| {
                                evt.stop_propagation();
                                consume_context::<MenuContext>().set_new_note_visible(Some(format!("{}/", prefix.clone())));
                            }
                        },
                        Icon { icon: LucideIcon::Plus, size: 18 }
                    }
                }
            }


            div { class: "ml-6", {children} }
        } else {
            button {
                class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
                onclick: toggle_open,
                Icon { icon: LucideIcon::Folder, size: 18 }
                span { class: "font-medium",
                    class: if is_footnote { "text-amber-400" } else { "text-zinc-200" },
                    "{name}"
                }
            }
        }
    }
}

#[component]
fn BrowserRowFile(node: TreeNode, is_footnote: bool) -> Element {
    let path_clone = node.full_path.clone();

    let onclick = move |_| {
        if let Some(relative_path) = &path_clone {
            consume_context::<MenuContext>().go_note(&relative_path.to_string_lossy().to_string());
        }
    };

    rsx! {
        button {
            class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
            onclick: onclick,
            Icon { icon: LucideIcon::FileText, size: 18 }
            span {
                class: if is_footnote { "text-amber-300" } else { "text-zinc-300" },
                "{node.name}"
            }
        }
    }
}
