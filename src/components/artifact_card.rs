use leptos::*;
use crate::models::Artifact;

#[component]
pub fn ArtifactCard(artifact: Artifact) -> impl IntoView {
    view! {
        <div class="group relative flex flex-col overflow-hidden rounded-xl bg-slate-800/50 border border-white/5 hover:border-cyan-500/50 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg hover:shadow-cyan-500/20">
            <div class="p-6 flex-grow space-y-4">
                // Icon Header
                <div class="flex items-center justify-between">
                    <div class="p-3 rounded-lg bg-slate-700/50 text-cyan-400 group-hover:text-white group-hover:bg-cyan-500 transition-colors duration-300">
                         // Render icon as raw HTML/SVG
                        <div inner_html=artifact.icon class="w-6 h-6"></div>
                    </div>

                    // Tags
                    <div class="flex flex-wrap gap-2 justify-end">
                        {artifact.tags.into_iter().map(|tag| view! {
                            <span class="px-2 py-1 text-[10px] uppercase tracking-wider font-bold rounded-full bg-white/5 text-slate-400 border border-white/5">
                                {tag}
                            </span>
                        }).collect_view()}
                    </div>
                </div>

                // Content
                <div>
                    <h3 class="text-xl font-bold text-white mb-2 group-hover:text-cyan-400 transition-colors">
                        {artifact.title}
                    </h3>
                    <p class="text-slate-400 text-sm leading-relaxed">
                        {artifact.description}
                    </p>
                </div>
            </div>

            // Action Footer
            {move || match &artifact.link {
                Some(url) => view! {
                    <div class="p-4 border-t border-white/5 bg-white/5">
                        <a href=url.clone() target="_blank" class="flex items-center justify-center w-full text-sm font-semibold text-cyan-400 group-hover:text-white transition-colors">
                            {artifact.link_text.clone()}
                            <span class="ml-2">"→"</span>
                        </a>
                    </div>
                }.into_view(),
                None => view! {}.into_view()
            }}
        </div>
    }
}