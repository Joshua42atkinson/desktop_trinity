use leptos::*;
use crate::components::glass_panel::GlassPanel;

#[component]
pub fn Planning() -> impl IntoView {
    view! {
        <div class="max-w-5xl mx-auto w-full space-y-10 animate-fade-in">
            <div class="text-center space-y-6">
                <h1 class="text-4xl font-bold text-white">"Planning & Analysis"</h1>
                <p class="text-slate-400">"Gap analysis, target population assessment, and technology selection."</p>
            </div>
            <GlassPanel>
                <p class="text-gray-300">"Content for Planning & Analysis goes here."</p>
            </GlassPanel>
        </div>
    }
}