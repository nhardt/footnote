html { class: "dark", lang: "en",
    head {
        meta { charset: "UTF-8" }
        meta {
            content: "width=device-width, initial-scale=1.0",
            name: "viewport",
        }
        title { "Footnote.wiki - Contacts" }
        script { src: "https://cdn.tailwindcss.com" }
        link { href: "https://fonts.googleapis.com", rel: "preconnect" }
        link {
            crossorigin: "false",
            href: "https://fonts.gstatic.com",
            rel: "preconnect",
        }
        link {
            href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=IBM+Plex+Sans:wght@400;500;600&display=swap",
            rel: "stylesheet",
        }
        script {
            "tailwind.config = {\n                darkMode: \"class\",\n                theme: {\n                    extend: {\n                        fontFamily: {\n                            sans: [\"IBM Plex Sans\", \"system-ui\", \"sans-serif\"],\n                            mono: [\"JetBrains Mono\", \"monospace\"],\n                        },\n                    },\n                },\n            };"
        }
        style {
            "* {\n                transition:\n                    border-color 150ms ease,\n                    background-color 150ms ease,\n                    box-shadow 150ms ease;\n            }\n\n            input:focus,\n            textarea:focus,\n            button:focus-visible {\n                outline: none;\n            }"
        }
    }
    body { class: "bg-zinc-950 text-zinc-100 font-sans antialiased",
        nav { class: "border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm",
            div { class: "px-6 py-3",
                div { class: "flex items-center gap-8",
                    button { class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                        "\n                        Notes\n                    "
                    }
                    button { class: "px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100 transition-colors",
                        "\n                        Profile\n                    "
                    }
                    button { class: "px-4 py-2 text-sm font-medium text-zinc-100 border-b-2 border-zinc-100",
                        "\n                        Contacts\n                    "
                    }
                    div { class: "ml-auto flex items-center gap-2",
                        div { class: "h-2 w-2 rounded-full bg-zinc-500" }
                        span { class: "text-sm font-mono text-zinc-400", "Sync Inactive" }
                    }
                }
            }
        }
        main { class: "max-w-3xl mx-auto px-6 py-12",
            div { class: "mb-8",
                h1 { class: "text-3xl font-bold font-mono mb-2", "Contacts" }
                p { class: "text-sm text-zinc-500",
                    "\n                    Manage your trust network for sharing notes\n                "
                }
            }
            section { class: "mb-8",
                button { class: "w-full px-6 py-4 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 rounded-lg text-sm font-medium text-zinc-300 hover:text-zinc-100 transition-all text-left",
                    div { class: "flex items-center justify-between",
                        div {
                            div { class: "font-semibold mb-1",
                                "\n                                Import Contact Record\n                            "
                            }
                            div { class: "text-xs text-zinc-500",
                                "\n                                Add a friend to your trust network\n                            "
                            }
                        }
                        svg {
                            class: "w-5 h-5 text-zinc-500",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path { d: "M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z" }
                        }
                    }
                }
            }
            section { class: "space-y-2",
                div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                    button { class: "w-full px-6 py-4 hover:bg-zinc-900/50 transition-colors text-left",
                        div { class: "flex items-center justify-between",
                            div { class: "flex-1",
                                div { class: "font-semibold mb-1", "nate-ui" }
                                div { class: "text-sm text-zinc-500", "nh" }
                            }
                            div { class: "flex items-center gap-4",
                                span { class: "text-sm text-zinc-500", "3 devices" }
                                svg {
                                    class: "w-5 h-5 text-zinc-500",
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
                            }
                        }
                    }
                }
                div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                    button { class: "w-full px-6 py-4 bg-zinc-900/50 transition-colors text-left",
                        div { class: "flex items-center justify-between",
                            div { class: "flex-1",
                                div { class: "font-semibold mb-1", "alice" }
                                div { class: "text-sm text-zinc-500",
                                    "\n                                    alice@example.com\n                                "
                                }
                            }
                            div { class: "flex items-center gap-4",
                                span { class: "text-sm text-zinc-500", "2 devices" }
                                svg {
                                    class: "w-5 h-5 text-zinc-400 transform rotate-90",
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
                            }
                        }
                    }
                    div { class: "px-6 pb-4 bg-zinc-900/20 border-t border-zinc-800",
                        div { class: "space-y-2 pt-4",
                            div { class: "flex items-center justify-between text-sm py-2",
                                span { class: "text-zinc-300", "laptop" }
                                span { class: "text-xs font-mono text-zinc-500 truncate ml-4",
                                    "121bae4696cd827d18c431547f343083a43..."
                                }
                            }
                            div { class: "flex items-center justify-between text-sm py-2",
                                span { class: "text-zinc-300", "phone" }
                                span { class: "text-xs font-mono text-zinc-500 truncate ml-4",
                                    "07b87ce9aa468bc8af9627e581d305ca2722..."
                                }
                            }
                        }
                    }
                }
            }
        }
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50",
            id: "import-modal",
            style: "display: none",
            div {
                class: "bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl max-w-2xl w-full max-h-[90vh] flex flex-col",
                "onclick": "event.stopPropagation()",
                div { class: "p-6 border-b border-zinc-800",
                    h3 { class: "text-lg font-semibold font-mono",
                        "\n                        Import Contact\n                    "
                    }
                    p { class: "text-sm text-zinc-500 mt-1",
                        "\n                        Add someone to your trust network\n                    "
                    }
                }
                div { class: "p-6 flex-1 min-h-0 flex flex-col gap-4",
                    div {
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "\n                            Nickname\n                            "
                            span { class: "text-zinc-500 font-normal ml-1",
                                "(how you'll reference them when sharing)"
                            }
                        }
                        input {
                            class: "w-full px-3 py-2 bg-zinc-950 border border-zinc-700 rounded-md text-sm font-mono focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            placeholder: "alice",
                            r#type: "text",
                        }
                    }
                    div { class: "flex-1 min-h-0 flex flex-col",
                        label { class: "block text-sm font-medium text-zinc-300 mb-2",
                            "Contact Record"
                        }
                        textarea {
                            class: "flex-1 w-full px-4 py-3 bg-zinc-950 border border-zinc-700 rounded-lg text-xs font-mono text-zinc-300 resize-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500",
                            placeholder: "{\"nickname\": \"@alice\", \"username\": \"alice\", ...}",
                        }
                    }
                    div {
                        class: "text-sm text-red-400 font-mono",
                        style: "display: none",
                        "\n                        Failed to import: Invalid JSON format\n                    "
                    }
                    div { class: "flex gap-3",
                        button { class: "flex-1 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600 rounded-md text-sm font-medium transition-all",
                            "\n                            Cancel\n                        "
                        }
                        button { class: "flex-1 px-4 py-2 bg-zinc-100 hover:bg-white hover:shadow-lg text-zinc-900 rounded-md text-sm font-medium transition-all",
                            "\n                            Import\n                        "
                        }
                    }
                }
            }
        }
    }
}
