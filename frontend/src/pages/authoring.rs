use crate::components::authoring::node_canvas::NodeCanvas;
use leptos::prelude::*;

#[component]
pub fn AuthoringPage() -> impl IntoView {
    view! {
        <div class="h-screen w-screen flex flex-col">
            // Header
            <header class="h-16 bg-slate-900 border-b border-white/10 flex items-center px-6 justify-between shrink-0">
                <div class="flex items-center gap-4">
                    <h1 class="text-xl font-bold text-white tracking-widest">"DAYDREAM" <span class="text-cyan-400">"AUTHOR"</span></h1>
                </div>
                <div class="flex items-center gap-4">
                    <span class="text-slate-500 text-sm">"v0.1.0-alpha"</span>
                </div>
            </header>

            // Main Content
            <main class="flex-grow relative">
                <NodeCanvas />
            </main>
        </div>
    }
}
