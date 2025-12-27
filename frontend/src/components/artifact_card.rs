use crate::models::Artifact;
use leptos::prelude::*;

#[component]
pub fn ArtifactCard(artifact: Artifact) -> impl IntoView {
    let state = use_context::<crate::ui_state::GlobalUiState>().expect("GlobalUiState missing");
    let tags = artifact.tags.clone();

    view! {
        <div class="group relative flex flex-col h-full overflow-hidden rounded-xl bg-slate-800/50 border border-white/5 hover:border-cyan-500/50 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg hover:shadow-cyan-500/20">
            <div class="p-6 flex-grow space-y-4">
                // Header with Icon and Tags
                <div class="flex items-start justify-between">
                    <div class="p-3 rounded-lg bg-slate-700/50 text-cyan-400 group-hover:text-white group-hover:bg-cyan-500 transition-colors duration-300">
                        <div inner_html=artifact.icon class="w-6 h-6"></div>
                    </div>

                    <div class="flex flex-wrap gap-2 justify-end">
                        {artifact.tags.into_iter().map(|tag| view! {
                            <span class="px-2 py-1 text-[10px] uppercase tracking-wider font-bold rounded-full bg-white/5 text-slate-400 border border-white/5">
                                {tag}
                            </span>
                        }).collect_view()}
                    </div>
                </div>

                // Title and Description
                <div>
                    <h3 class="text-xl font-bold text-white mb-2 group-hover:text-cyan-400 transition-colors">
                        {artifact.title}
                    </h3>
                    <p class="text-slate-400 text-sm leading-relaxed">
                        {artifact.description}
                    </p>
                </div>
            </div>

            // Link Footer
            {move || match &artifact.link {
                Some(url) => {
                    let url = url.clone();
                    let tags_for_click = tags.clone();
                    view! {
                    <div class="p-4 border-t border-white/5 bg-white/5 mt-auto">
                        <button
                            class="flex items-center justify-center w-full text-sm font-semibold text-cyan-400 group-hover:text-white transition-colors"
                            on:click=move |ev| {
                                ev.prevent_default();
                                if tags_for_click.contains(&"Video".to_string()) {
                                    state.open_hud(crate::ui_state::HudContent::Video(url.clone()));
                                } else if tags_for_click.contains(&"Code".to_string()) {
                                    // Placeholder for code content fetching - in real app we'd fetch the raw file
                                    state.open_hud(crate::ui_state::HudContent::Code(format!("// Content of {}\nfn main() {{}}", url), "rust".to_string()));
                                } else {
                                    // Default fallback logic or open in new tab
                                    let _ = window().open_with_url_and_target(&url, "_blank");
                                }
                            }
                        >
                            {artifact.link_text.clone()}
                            <span class="ml-2">"→"</span>
                        </button>
                    </div>
                }.into_any()},
                None => ().into_any()
            }}
        </div>
    }
}
