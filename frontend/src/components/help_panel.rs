use leptos::ev::MouseEvent;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn HelpPanel(
    /// Whether the help panel is visible
    #[prop(into)]
    show: Signal<bool>,
    /// Callback to close the panel
    on_close: Callback<()>,
    /// Title of the help panel
    #[prop(into)]
    title: String,
    /// Content as string
    #[prop(into)]
    content: String,
) -> impl IntoView {
    let handle_backdrop_click = move |ev: MouseEvent| {
        if ev
            .target()
            .map(|t| {
                t.dyn_into::<web_sys::HtmlElement>()
                    .ok()
                    .and_then(|el| el.get_attribute("data-backdrop"))
                    .is_some()
            })
            .unwrap_or(false)
        {
            on_close.run(());
        }
    };

    view! {
        <Show when=move || show.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in"
                on:click=handle_backdrop_click
                data-backdrop="true"
            >
                <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>

                <div class="relative z-10 max-w-2xl w-full max-h-[80vh] overflow-hidden rounded-2xl border border-white/20 bg-slate-900/90 backdrop-blur-xl shadow-2xl animate-slide-up">
                    <div class="flex items-center justify-between p-6 border-b border-white/10">
                        <h2 class="text-2xl font-bold text-white">{title}</h2>
                        <button
                            on:click=move |_| on_close.run(())
                            class="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-400 hover:text-white"
                            aria-label="Close help panel"
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>

                    <div class="p-6 overflow-y-auto max-h-[calc(80vh-5rem)] custom-scrollbar text-slate-300 whitespace-pre-wrap">
                        {content}
                    </div>

                    <div class="p-4 border-t border-white/10 bg-slate-900/50 flex justify-end">
                        <div class="text-sm text-slate-400">
                            "Press " <kbd class="px-2 py-1 bg-white/10 rounded border border-white/20 font-mono text-xs">"?"</kbd> " to toggle help"
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
