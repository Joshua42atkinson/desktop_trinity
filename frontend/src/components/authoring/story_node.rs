use common::expert::StoryNode;
use leptos::prelude::*;

#[component]
pub fn StoryNodeComponent(
    #[prop(into)] node: RwSignal<StoryNode>,
    #[prop(into)] on_mousedown: Callback<web_sys::MouseEvent>,
    #[prop(into)] on_port_mousedown: Callback<(String, String)>, // (node_id, "input" | "output")
    #[prop(into)] on_port_mouseup: Callback<(String, String)>,   // (node_id, "input" | "output")
) -> impl IntoView {
    // We subscribe to the signal to get updates
    let node_data = move || node.get();

    view! {
        <div
            class="absolute bg-slate-800 border-2 border-cyan-500/50 rounded-lg w-64 shadow-lg cursor-move hover:border-cyan-400 transition-colors select-none"
            style=move || format!("left: {}px; top: {}px;", node_data().x, node_data().y)
            on:mousedown=move |ev| on_mousedown.run(ev)
        >
            // Header
            <div class="bg-slate-900/50 p-2 border-b border-white/10 flex justify-between items-center rounded-t-lg">
                <span class="font-bold text-cyan-400 text-sm truncate">{move || node_data().title}</span>
                <div class="w-2 h-2 rounded-full bg-cyan-500 shadow-[0_0_10px_rgba(6,182,212,0.5)]"></div>
            </div>

            // Content Preview
            <div class="p-3 text-slate-300 text-xs font-mono h-24 overflow-hidden relative">
                {move || node_data().content}
                <div class="absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-slate-800 to-transparent"></div>
            </div>

            // Input Port (Left)
            <div
                class="absolute -left-3 top-1/2 -translate-y-1/2 w-4 h-4 bg-slate-900 border border-cyan-500 rounded-full hover:bg-cyan-500 transition-colors cursor-crosshair"
                on:mousedown=move |ev| { ev.stop_propagation(); on_port_mousedown.run((node_data().id.clone(), "input".to_string())); }
                on:mouseup=move |ev| { ev.stop_propagation(); on_port_mouseup.run((node_data().id.clone(), "input".to_string())); }
            ></div>

            // Output Port (Right)
            <div
                class="absolute -right-3 top-1/2 -translate-y-1/2 w-4 h-4 bg-slate-900 border border-cyan-500 rounded-full hover:bg-cyan-500 transition-colors cursor-crosshair"
                on:mousedown=move |ev| { ev.stop_propagation(); on_port_mousedown.run((node_data().id.clone(), "output".to_string())); }
                on:mouseup=move |ev| { ev.stop_propagation(); on_port_mouseup.run((node_data().id.clone(), "output".to_string())); }
            ></div>
        </div>
    }
}
