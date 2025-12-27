use crate::api::{get_graph, save_graph};
use crate::components::authoring::property_editor::PropertyEditor;
use crate::components::authoring::story_node::StoryNodeComponent;
use common::expert::{Connection, StoryGraph, StoryNode};
use leptos::prelude::*;

#[component]
pub fn NodeCanvas() -> impl IntoView {
    let (nodes, set_nodes) = signal(Vec::<RwSignal<StoryNode>>::new());
    let (connections, set_connections) = signal(Vec::<Connection>::new());
    let (graph_meta, set_graph_meta) = signal((String::new(), String::new())); // id, title
    let (dragging_id, set_dragging_id) = signal(None::<String>);
    let (selected_node_id, set_selected_node_id) = signal(None::<String>);
    let (offset, set_offset) = signal((0.0, 0.0)); // Mouse offset relative to node top-left

    // Connection dragging state
    let (connecting_source, set_connecting_source) = signal(None::<String>); // node_id
    let (mouse_pos, set_mouse_pos) = signal((0.0, 0.0)); // For drawing the temp line

    // Load graph on mount
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            match get_graph().await {
                Ok(graph) => {
                    let node_signals = graph.nodes.into_iter().map(|n| RwSignal::new(n)).collect();
                    set_nodes.set(node_signals);
                    set_connections.set(graph.connections);
                    set_graph_meta.set((graph.id, graph.title));
                }
                Err(e) => leptos::logging::error!("Failed to load graph: {}", e),
            }
        });
    });

    // Save graph handler
    let save_graph_handler = move |_| {
        leptos::task::spawn_local(async move {
            let current_nodes: Vec<StoryNode> = nodes.get().iter().map(|s| s.get()).collect();
            let current_connections = connections.get();
            let (id, title) = graph_meta.get();
            let graph = StoryGraph {
                id: if id.is_empty() {
                    "demo_graph".to_string()
                } else {
                    id
                },
                title: if title.is_empty() {
                    "New Story".to_string()
                } else {
                    title
                },
                nodes: current_nodes,
                connections: current_connections,
            };

            match save_graph(graph).await {
                Ok(_) => leptos::logging::log!("Graph saved successfully"),
                Err(e) => leptos::logging::error!("Failed to save graph: {}", e),
            }
        });
    };

    let on_mouse_move = move |ev: web_sys::MouseEvent| {
        let mx = ev.client_x() as f64;
        let my = ev.client_y() as f64;
        set_mouse_pos.set((mx, my));

        if let Some(id) = dragging_id.get() {
            // Find the node being dragged
            if let Some(node_signal) = nodes.get().iter().find(|n| n.get().id == id) {
                node_signal.update(|n| {
                    n.x = mx - offset.get().0;
                    n.y = my - offset.get().1;
                });
            }
        }
    };

    let on_mouse_up = move |_| {
        set_dragging_id.set(None);
        set_connecting_source.set(None);
    };

    // Helper to start dragging
    let start_drag =
        move |ev: web_sys::MouseEvent, node_id: String, initial_x: f64, initial_y: f64| {
            set_dragging_id.set(Some(node_id.clone()));
            set_selected_node_id.set(Some(node_id));
            set_offset.set((
                ev.client_x() as f64 - initial_x,
                ev.client_y() as f64 - initial_y,
            ));
        };

    // Port handlers
    let on_port_mousedown = move |(node_id, port_type): (String, String)| {
        if port_type == "output" {
            set_connecting_source.set(Some(node_id));
        }
    };

    let on_port_mouseup = move |(node_id, port_type): (String, String)| {
        if let Some(source_id) = connecting_source.get() {
            if port_type == "input" && source_id != node_id {
                // Create connection
                set_connections.update(|c| {
                    c.push(Connection {
                        id: uuid::Uuid::new_v4().to_string(),
                        from_node: source_id,
                        to_node: node_id,
                    });
                });
                set_connecting_source.set(None);
            }
        }
    };

    // Helper to get node position by ID
    // We need to access nodes signal, so this should be used inside a reactive context
    let get_node_pos = move |id: &str| -> Option<(f64, f64)> {
        nodes.get().iter().find(|n| n.get().id == id).map(|n| {
            let d = n.get();
            (d.x, d.y)
        })
    };

    view! {
        <div class="relative w-full h-full bg-slate-950 overflow-hidden"
             style="background-image: radial-gradient(#334155 1px, transparent 1px); background-size: 20px 20px;"
             on:mousemove=on_mouse_move
             on:mouseup=on_mouse_up
             on:mouseleave=on_mouse_up
        >
            // Toolbar
            <div class="absolute top-4 right-4 z-10 flex gap-2">
                <button
                    class="px-4 py-2 bg-slate-800 hover:bg-slate-700 border border-white/10 text-white rounded shadow-lg font-bold transition-colors"
                    on:click=move |_| {
                        let new_node = StoryNode {
                            id: uuid::Uuid::new_v4().to_string(),
                            title: "New Node".to_string(),
                            content: "Write something...".to_string(),
                            x: 100.0,
                            y: 100.0,
                        };
                        set_nodes.update(|n| n.push(RwSignal::new(new_node)));
                    }
                >
                    "+ Add Node"
                </button>
                <button
                    class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded shadow-lg font-bold transition-colors"
                    on:click=save_graph_handler
                >
                    "Save Graph"
                </button>
            </div>

            // SVG Layer for Connections
            <svg class="absolute inset-0 w-full h-full pointer-events-none z-0">
                <defs>
                    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                        <polygon points="0 0, 10 3.5, 0 7" fill="#06b6d4" />
                    </marker>
                </defs>
                <For
                    each=move || connections.get()
                    key=|c| c.id.clone()
                    children=move |c| {
                        let from_pos = get_node_pos(&c.from_node).unwrap_or((0.0, 0.0));
                        let to_pos = get_node_pos(&c.to_node).unwrap_or((0.0, 0.0));
                        // Calculate port positions (Output: Right center, Input: Left center)
                        // Node width 256px (w-64), Height approx 100px (header + content)
                        // Let's assume input is at (x, y+50), output at (x+256, y+50)
                        let x1 = from_pos.0 + 256.0;
                        let y1 = from_pos.1 + 50.0;
                        let x2 = to_pos.0;
                        let y2 = to_pos.1 + 50.0;

                        // Bezier Control Points
                        // Control point 1: x1 + curvature, y1
                        // Control point 2: x2 - curvature, y2
                        let curvature = 0.5 * (x2 - x1).abs().max(100.0);
                        let cp1_x = x1 + curvature;
                        let cp1_y = y1;
                        let cp2_x = x2 - curvature;
                        let cp2_y = y2;

                        let path_d = format!(
                            "M {} {} C {} {}, {} {}, {} {}",
                            x1, y1, cp1_x, cp1_y, cp2_x, cp2_y, x2, y2
                        );

                        view! {
                            <path d=path_d stroke="#06b6d4" stroke-width="2" fill="none" marker-end="url(#arrowhead)" />
                        }
                    }
                />

                // Temp Connection Line
                {move || if let Some(source_id) = connecting_source.get() {
                    let from_pos = get_node_pos(&source_id).unwrap_or((0.0, 0.0));
                    let x1 = from_pos.0 + 256.0;
                    let y1 = from_pos.1 + 50.0;
                    let (x2, y2) = mouse_pos.get();
                    view! {
                        <line x1=x1 y1=y1 x2=x2 y2=y2 stroke="#06b6d4" stroke-width="2" stroke-dasharray="5,5" />
                    }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }}
            </svg>

            // Canvas Area
            <div class="absolute inset-0 z-10">
                <For
                    each=move || nodes.get()
                    key=|node| node.get().id
                    children=move |node| {
                        let id = node.get().id.clone();
                        view! {
                            <StoryNodeComponent
                                node=node
                                on_mousedown=move |ev| {
                                    let n = node.get();
                                    start_drag(ev, id.clone(), n.x, n.y);
                                }
                                on_port_mousedown=on_port_mousedown
                                on_port_mouseup=on_port_mouseup
                            />
                        }
                    }
                />
            </div>

            // Instructions if empty
            {move || if nodes.get().is_empty() {
                view! {
                    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <p class="text-slate-500">"Loading graph or empty..."</p>
                    </div>
                }.into_any()
            } else {
                view! { <span /> }.into_any()
            }}

            // Property Editor
            {move || selected_node_id.get().and_then(|id| {
                nodes.get().iter().find(|n| n.get().id == id).cloned().map(|node_signal| {
                    view! {
                        <PropertyEditor
                            node=node_signal
                            on_close=move |_| set_selected_node_id.set(None)
                            on_delete=move |_| {
                                set_nodes.update(|n| n.retain(|x| x.get().id != id));
                                set_connections.update(|c| c.retain(|x| x.from_node != id && x.to_node != id));
                                set_selected_node_id.set(None);
                            }
                        />
                    }
                })
            })}
        </div>
    }
}
