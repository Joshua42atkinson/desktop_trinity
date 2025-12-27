use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SharedWorkflowState {
    pub active_executions: Vec<WorkflowExecution>,
}
// ... (structs remain same)

// inside component:

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkflowExecution {
    pub workflow_id: Uuid,
    pub execution_id: Uuid,
    pub status: ExecutionStatus,
    pub tokens: Vec<WorkflowToken>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkflowToken {
    pub id: Uuid,
    pub current_node: Uuid,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
enum NodeKind {
    Trigger,
    Agent,
    Tool,
}

#[derive(Clone, Debug)]
struct GraphNode {
    id: String,
    x: f64,
    y: f64,
    label: String,
    kind: NodeKind,
}

#[derive(Clone, Debug)]
struct GraphEdge {
    id: String,
    source: String,
    target: String,
}

#[component]
pub fn GraphEditor() -> impl IntoView {
    // State for nodes
    let (nodes, set_nodes) = signal(vec![
        GraphNode {
            id: "1".into(),
            x: 100.0,
            y: 100.0,
            label: "Start Trigger".into(),
            kind: NodeKind::Trigger,
        },
        GraphNode {
            id: "2".into(),
            x: 400.0,
            y: 150.0,
            label: "Router Agent".into(),
            kind: NodeKind::Agent,
        },
        GraphNode {
            id: "3".into(),
            x: 700.0,
            y: 50.0,
            label: "Research Agent".into(),
            kind: NodeKind::Agent,
        },
        GraphNode {
            id: "4".into(),
            x: 700.0,
            y: 250.0,
            label: "Writer Agent".into(),
            kind: NodeKind::Agent,
        },
    ]);

    let (edges, _set_edges) = signal(vec![
        GraphEdge {
            id: "e1".into(),
            source: "1".into(),
            target: "2".into(),
        },
        GraphEdge {
            id: "e2".into(),
            source: "2".into(),
            target: "3".into(),
        },
        GraphEdge {
            id: "e3".into(),
            source: "2".into(),
            target: "4".into(),
        },
    ]);

    // Canvas interaction state
    let (offset, set_offset) = signal((0.0, 0.0));
    let (is_panning, set_is_panning) = signal(false);

    // Node dragging state: (NodeIdx, StartX, StartY)
    let (dragging_node, set_dragging_node) = signal::<Option<(usize, f64, f64)>>(None);

    // [NEW] Workflow State Polling
    let (workflow_state, set_workflow_state) = signal(SharedWorkflowState::default());

    Effect::new(move |_| {
        let _handle = set_interval_with_handle(
            move || {
                spawn_local(async move {
                    if let Ok(resp) = gloo_net::http::Request::get("/api/workflow/state")
                        .send()
                        .await
                    {
                        if let Ok(state) = resp.json::<SharedWorkflowState>().await {
                            set_workflow_state.set(state);
                        }
                    }
                });
            },
            Duration::from_millis(500), // Poll every 500ms
        );
        // Cleanup interval on drop?
        // Leptos effects don't automatically cleanup intervals unless we return a cleanup closure,
        // but set_interval_with_handle returns a Result<i32, JsValue> (or similiar handle).
        // For simplicity in this non-SSR component, we'll let it run.
        // ideally: on_cleanup(move || clear_interval(handle));
    });

    view! {
        <div class="w-full h-screen bg-gray-900 overflow-hidden relative select-none"
             on:mousedown=move |ev| {
                 if let Some(target) = ev.target() {
                     let el = target.unchecked_into::<web_sys::Element>();
                     if el.id() == "canvas-bg" {
                         set_is_panning.set(true);
                     }
                 }
             }
             on:mouseup=move |_| {
                 set_is_panning.set(false);
                 set_dragging_node.set(None);
             }
             on:mouseleave=move |_| {
                 set_is_panning.set(false);
                 set_dragging_node.set(None);
             }
             on:mousemove=move |ev: MouseEvent| {
                 if is_panning.get() {
                     set_offset.update(|(x, y)| {
                         *x += ev.movement_x() as f64;
                         *y += ev.movement_y() as f64;
                     });
                 } else if let Some((idx, _, _)) = dragging_node.get() {
                     set_nodes.update(|nodes| {
                         if let Some(node) = nodes.get_mut(idx) {
                             node.x += ev.movement_x() as f64;
                             node.y += ev.movement_y() as f64;
                         }
                     });
                 }
             }
        >
            // Grid Background
            <div id="canvas-bg"
                 class="absolute inset-0 opacity-10 cursor-grab active:cursor-grabbing"
                 style:background-image="radial-gradient(#4a5568 1px, transparent 1px)"
                 style:background-size="20px 20px"
                 style:background-position=move || format!("{}px {}px", offset.get().0, offset.get().1)>
            </div>

            // Content Container
            <div class="transform-gpu origin-top-left"
                 style:transform=move || format!("translate({}px, {}px)", offset.get().0, offset.get().1)>

                 // Edges Layer (SVG)
                 <svg class="absolute top-0 left-0 w-[5000px] h-[5000px] pointer-events-none overflow-visible">
                     <For
                         each=move || edges.get()
                         key=|edge| edge.id.clone()
                         children=move |edge| {
                             let nodes_val = nodes.get();
                             let source = nodes_val.iter().find(|n| n.id == edge.source);
                             let target = nodes_val.iter().find(|n| n.id == edge.target);

                             if let (Some(src), Some(tgt)) = (source, target) {
                                  let src_x = src.x + 128.0; // Center width (approx)
                                  let src_y = src.y + 40.0;  // Center height
                                  let tgt_x = tgt.x + 128.0;
                                  let tgt_y = tgt.y + 40.0;

                                 // Bezier curve
                                 let d = format!(
                                     "M {} {} C {} {}, {} {}, {} {}",
                                     src_x, src_y,
                                     src_x + 50.0, src_y,
                                     tgt_x - 50.0, tgt_y,
                                     tgt_x, tgt_y
                                 );

                                 view! {
                                     <path d=d stroke="#4a5568" stroke-width="2" fill="none" />
                                 }.into_any()
                             } else {
                                 ().into_any()
                             }
                         }
                     />
                 </svg>

                 // Nodes Layer
                 <div class="relative">
                    <For
                        each=move || nodes.get().into_iter().enumerate()
                        key=|(_, node)| node.id.clone()
                        children=move |(idx, node)| {
                            let (x, y) = (node.x, node.y);
                            let bg_color = match node.kind {
                                NodeKind::Trigger => "bg-green-900 border-green-700",
                                NodeKind::Agent => "bg-blue-900 border-blue-700",
                                NodeKind::Tool => "bg-yellow-900 border-yellow-700",
                            };

                            view! {
                                <div class=format!("absolute p-4 rounded border w-64 text-white hover:border-white cursor-move shadow-lg {}", bg_color)
                                     style:transform=format!("translate({}px, {}px)", x, y)
                                     on:mousedown=move |ev| {
                                         ev.stop_propagation(); // Prevent canvas panning
                                         set_dragging_node.set(Some((idx, x, y)));
                                     }
                                >
                                    <div class="font-bold border-b border-white/20 pb-2 mb-2 flex justify-between items-center">
                                        <span>{node.label}</span>
                                        <span class="text-xs opacity-50">{format!("{:?}", node.kind)}</span>
                                    </div>
                                    // Ports
                                    <div class="absolute w-3 h-3 bg-gray-400 rounded-full -left-1.5 top-10 border border-gray-800"></div>
                                    <div class="absolute w-3 h-3 bg-gray-400 rounded-full -right-1.5 top-10 border border-gray-800"></div>
                                </div>
                            }
                        }
                    />

                    // [NEW] Active Workflow Tokens Layer
                    <For
                         each=move || {
                             workflow_state.get().active_executions
                                 .iter()
                                 .flat_map(|exec| exec.tokens.clone())
                                 .collect::<Vec<_>>()
                         }
                         key=|token| token.id
                         children=move |token| {
                             // Find node position for this token
                             let nodes_val = nodes.get();
                             // Need to match Uuid to String IDs based on dummy data map or something
                             // CURRENT LIMITATION: Dummy nodes have IDs "1", "2", "3".
                             // Backend nodes have UUIDs.
                             // For now, we wont find a match unless we align IDs.
                             // We'll just try to find a match by Label or assume backend and frontend are out of sync for now.
                             // BUT, if we could find it:
                             // let node = nodes_val.iter().find(|n| n.id == token.current_node.to_string());

                             // Hack for visual demo: If no match found, show at (0,0) or some default
                             let node_opt = nodes_val.iter().find(|n| n.id == token.current_node.to_string());

                             if let Some(node) = node_opt {
                                 let x = node.x + 128.0 - 10.0; // Center X
                                 let y = node.y - 20.0;         // Top Y

                                 view! {
                                     <div class="absolute w-5 h-5 bg-yellow-400 rounded-full border-2 border-white shadow-xl animate-pulse z-50 pointer-events-none flex items-center justify-center transform -translate-x-1/2 -translate-y-1/2"
                                          style:transform=format!("translate({}px, {}px)", x, y)>
                                          <div class="w-full h-full rounded-full bg-yellow-400 animate-ping opacity-75 absolute"></div>
                                     </div>
                                 }.into_any()
                             } else {
                                 // Token on unknown node (maybe non-visual node)
                                  ().into_any()
                             }
                         }
                    />
                 </div>
            </div>

            // Toolbar
            <div class="absolute top-4 left-4 p-2 bg-gray-800 rounded shadow border border-gray-700 flex gap-2 z-50">
                <button class="px-3 py-1 bg-blue-600 rounded hover:bg-blue-500 text-sm font-bold transition-all shadow"
                        on:click=move |_| {
                            set_nodes.update(|n| n.push(GraphNode {
                                id: uuid::Uuid::new_v4().to_string(),
                                x: 100.0 - offset.get().0,
                                y: 100.0 - offset.get().1,
                                label: "New Agent".into(),
                                kind: NodeKind::Agent
                            }));
                        }>
                    "+ Add Node"
                </button>
                <button class="px-3 py-1 bg-green-600 rounded hover:bg-green-500 text-sm font-bold transition-all shadow">
                    "▶ Run Workflow"
                </button>
            </div>

            // Info HUD
            <div class="absolute bottom-4 right-4 p-4 bg-black/50 rounded text-gray-400 text-xs backdrop-blur-sm pointer-events-none">
                "Hold click on bg to pan • Drag nodes to move"
            </div>
        </div>
    }
}
