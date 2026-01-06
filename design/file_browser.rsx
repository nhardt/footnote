
    html { class: "dark", lang: "en",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "File Browser Popover" }
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
                "* {\n                transition:\n                    border-color 150ms ease,\n                    background-color 150ms ease,\n                    box-shadow 150ms ease;\n            }"
            }
        }
        body { class: "p-8 font-sans antialiased bg-zinc-950 text-zinc-100",
            div { class: "mx-auto max-w-5xl",
                div { class: "p-6 mb-4 rounded-lg border border-zinc-800 bg-zinc-900/30",
                    div { class: "flex gap-4 items-center mb-4",
                        label { class: "text-sm font-medium text-zinc-400", "File" }
                        button { class: "flex flex-1 justify-between items-center py-1.5 px-3 font-mono text-sm text-left rounded-md border transition-colors bg-zinc-900 border-zinc-700 group hover:border-zinc-500",
                            span { "projects/footnote/notes/architecture.md" }
                            svg {
                                class: "w-4 h-4 text-zinc-500 group-hover:text-zinc-400",
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
                        }
                    }
                }
                div { class: "relative",
                    div { class: "absolute top-0 right-0 left-0 z-50",
                        div { class: "overflow-y-auto rounded-lg border shadow-2xl border-zinc-700 bg-zinc-900 max-h-[500px]",
                            div { class: "sticky top-0 py-3 px-4 border-b bg-zinc-900 border-zinc-800",
                                div { class: "flex justify-between items-center",
                                    h3 { class: "text-sm font-semibold",
                                        "\n                                    Browse Files\n                                "
                                    }
                                    button { class: "p-1 rounded transition-colors text-zinc-500 hover:text-zinc-300",
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                d: "M6 18L18 6M6 6l12 12",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "p-2",
                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                    span { class: "font-medium", "inbox" }
                                }
                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                    span { class: "font-medium", "journal" }
                                }
                                div {
                                    button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                        span { class: "font-medium", "projects" }
                                    }
                                    div { class: "ml-6",
                                        div {
                                            button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                                span { class: "font-medium", "footnote" }
                                            }
                                            div { class: "ml-6",
                                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                                    span { class: "font-medium", "notes" }
                                                }
                                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors bg-zinc-800/50 hover:bg-zinc-800",
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
                                                    span { class: "text-zinc-300", "architecture.md" }
                                                }
                                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                                    span { class: "text-zinc-300", "roadmap.md" }
                                                }
                                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                                    span { class: "text-zinc-300", "todo.md" }
                                                }
                                            }
                                        }
                                        button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                            span { class: "font-medium", "website" }
                                        }
                                    }
                                }
                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                    span { class: "text-zinc-300", "home.md" }
                                }
                                button { class: "flex gap-2 items-center py-1.5 px-2 w-full text-sm text-left rounded transition-colors hover:bg-zinc-800",
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
                                    span { class: "text-zinc-300", "inbox.md" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }