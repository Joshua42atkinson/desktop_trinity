use leptos::*;
use crate::components::glass_panel::GlassPanel;

#[component]
pub fn Evaluation() -> impl IntoView {
    view! {
        <div class="max-w-5xl mx-auto w-full space-y-10 animate-fade-in">
            <div class="text-center space-y-6">
                <h1 class="text-4xl font-bold text-white">"Evaluation & Implementation"</h1>
                <p class="text-slate-400">"Formative/summative evaluation and implementation planning."</p>
            </div>
            <GlassPanel>
                <p class="text-gray-300">"Content for Evaluation & Implementation goes here."</p>
            </GlassPanel>
        </div>
    }
}