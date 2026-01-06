
    html { class: "dark", lang: "en",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "Device Row - Dashboard View" }
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
        body { class: "bg-zinc-950 text-zinc-100 font-sans antialiased p-8",
            div { class: "max-w-4xl mx-auto space-y-4",
                h2 { class: "text-xl font-bold font-mono mb-6", "Device Sync Status" }
                div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                    div { class: "p-4 hover:bg-zinc-900/50 transition-colors group",
                        div { class: "flex items-start justify-between mb-3",
                            div { class: "flex-1 min-w-0",
                                h3 { class: "text-sm font-semibold mb-1", "mac" }
                                p { class: "text-xs font-mono text-zinc-500 truncate",
                                    "\n                                0df2eb0375d0ce46b4e15490b135f342069aff70d21a7f7a0e781ebf0ed23d3d\n                            "
                                }
                            }
                            button { class: "opacity-0 group-hover:opacity-100 p-1.5 text-zinc-500 hover:text-red-400 hover:bg-zinc-800 rounded-md transition-all",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                    }
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-x-6 gap-y-2 text-xs",
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Outgoing\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-emerald-400 font-mono", "2m ago (3 files)" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-zinc-500 font-mono", "2m ago" }
                                }
                            }
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Incoming\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-emerald-400 font-mono", "5m ago (1 file)" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-zinc-500 font-mono", "5m ago" }
                                }
                            }
                        }
                    }
                }
                div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                    div { class: "p-4 hover:bg-zinc-900/50 transition-colors group",
                        div { class: "flex items-start justify-between mb-3",
                            div { class: "flex-1 min-w-0",
                                h3 { class: "text-sm font-semibold mb-1", "Phone" }
                                p { class: "text-xs font-mono text-zinc-500 truncate",
                                    "\n                                6640aa2155ccc361906efe283363f73a7acf9d9c5e5e2a551b1202d81cdcc1c7\n                            "
                                }
                            }
                            button { class: "opacity-0 group-hover:opacity-100 p-1.5 text-zinc-500 hover:text-red-400 hover:bg-zinc-800 rounded-md transition-all",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                    }
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-x-6 gap-y-2 text-xs",
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Outgoing\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-zinc-500 font-mono", "3h ago (5 files)" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-red-400 font-mono", "12m ago" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-zinc-500 font-mono", "12m ago" }
                                }
                            }
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Incoming\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-emerald-400 font-mono", "8m ago (2 files)" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-zinc-500 font-mono", "8m ago" }
                                }
                            }
                        }
                    }
                }
                div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                    div { class: "p-4 hover:bg-zinc-900/50 transition-colors group",
                        div { class: "flex items-start justify-between mb-3",
                            div { class: "flex-1 min-w-0",
                                h3 { class: "text-sm font-semibold mb-1",
                                    "\n                                secondary\n                            "
                                }
                                p { class: "text-xs font-mono text-zinc-500 truncate",
                                    "\n                                3f727fb5e22ff4a8e446464931011d1eba3460f4454d883a871d0410fae4ecfd\n                            "
                                }
                            }
                            button { class: "opacity-0 group-hover:opacity-100 p-1.5 text-zinc-500 hover:text-red-400 hover:bg-zinc-800 rounded-md transition-all",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                    }
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-x-6 gap-y-2 text-xs",
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Outgoing\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-zinc-500 font-mono", "Jan 3 (12 files)" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-amber-500 font-mono", "Jan 3" }
                                }
                            }
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Incoming\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-zinc-500 font-mono", "Jan 3 (0 files)" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-amber-500 font-mono", "Jan 3" }
                                }
                            }
                        }
                    }
                }
                div { class: "border border-zinc-800 rounded-lg bg-zinc-900/30 overflow-hidden",
                    div { class: "p-4 hover:bg-zinc-900/50 transition-colors group",
                        div { class: "flex items-start justify-between mb-3",
                            div { class: "flex-1 min-w-0",
                                h3 { class: "text-sm font-semibold mb-1", "tablet" }
                                p { class: "text-xs font-mono text-zinc-500 truncate",
                                    "\n                                a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2\n                            "
                                }
                            }
                            button { class: "opacity-0 group-hover:opacity-100 p-1.5 text-zinc-500 hover:text-red-400 hover:bg-zinc-800 rounded-md transition-all",
                                svg {
                                    class: "w-4 h-4",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                    }
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-x-6 gap-y-2 text-xs",
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Outgoing\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                            }
                            div { class: "space-y-1.5",
                                div { class: "text-zinc-500 font-semibold mb-2",
                                    "\n                                Incoming\n                            "
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last success" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last failure" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                                div { class: "flex justify-between items-baseline",
                                    span { class: "text-zinc-400", "Last seen" }
                                    span { class: "text-zinc-600 font-mono", "—" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }