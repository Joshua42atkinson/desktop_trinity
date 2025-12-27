use leptos::prelude::*;

/// Simple tutorial overlay - shows multi-step guidance
#[component]
pub fn TutorialOverlay(
    /// Whether to show the tutorial
    #[prop(into)]
    show: Signal<bool>,
    /// Callback to close
    on_close: Callback<()>,
    /// Tutorial title
    #[prop(into)]
    title: String,
    /// Tutorial steps (simple strings for now)
    #[prop(into)]
    steps: Vec<String>,
) -> impl IntoView {
    let (current_step, set_current_step) = signal(0usize);
    let total_steps = steps.len();

    let go_next = move || {
        let next = current_step.get() + 1;
        if next < total_steps {
            set_current_step.set(next);
        } else {
            on_close.run(());
        }
    };

    let go_prev = move || {
        if current_step.get() > 0 {
            set_current_step.set(current_step.get() - 1);
        }
    };

    let show_back = move || current_step.get() > 0;

    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 z-50 animate-fade-in">
                <div class="absolute inset-0 bg-black/80 backdrop-blur-sm"></div>

                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg mx-4">
                    <div class="bg-slate-900/95 backdrop-blur-xl rounded-2xl border border-white/20 shadow-2xl overflow-hidden">
                        <div class="p-6 border-b border-white/10 bg-gradient-to-r from-purple-900/30 to-cyan-900/30">
                            <div class="flex items-start justify-between">
                                <div class="flex-1">
                                    <div class="flex items-center gap-2 mb-2">
                                        <div class="w-2 h-2 bg-cyan-400 rounded-full animate-pulse"></div>
                                        <span class="text-xs font-bold text-cyan-400 uppercase tracking-wider">
                                            {title.clone()}
                                        </span>
                                    </div>
                                    <h2 class="text-2xl font-bold text-white">
                                        {move || format!("Step {} of {}", current_step.get() + 1, total_steps)}
                                    </h2>
                                </div>
                                <button
                                    on:click=move |_| on_close.run(())
                                    class="text-slate-400 hover:text-white transition-colors text-sm"
                                >
                                    "Skip"
                                </button>
                            </div>
                        </div>

                        <div class="p-6">
                            <p class="text-slate-300 text-lg leading-relaxed">
                                {move || steps.get(current_step.get()).cloned().unwrap_or_default()}
                            </p>
                        </div>

                        <div class="p-6 border-t border-white/10 bg-slate-900/50">
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-2">
                                    {(0..total_steps).map(|idx| {
                                        view! {
                                            <div class=move || {
                                                let current = current_step.get();
                                                if idx == current {
                                                    "w-8 h-2 bg-gradient-to-r from-purple-500 to-cyan-500 rounded-full"
                                                } else if idx < current {
                                                    "w-2 h-2 bg-white/40 rounded-full"
                                                } else {
                                                    "w-2 h-2 bg-white/20 rounded-full"
                                                }
                                            }></div>
                                        }
                                    }).collect_view()}
                                </div>

                                <div class="flex items-center gap-3">
                                    <Show when=show_back>
                                        <button
                                            on:click=move |_| go_prev()
                                            class="px-4 py-2 text-slate-400 hover:text-white transition-colors text-sm font-medium"
                                        >
                                            "← Back"
                                        </button>
                                    </Show>
                                    <button
                                        on:click=move |_| go_next()
                                        class="px-6 py-2 bg-gradient-to-r from-purple-600 to-cyan-600 hover:from-purple-500 hover:to-cyan-500 rounded-lg text-white font-bold transition-all duration-200 text-sm"
                                    >
                                        {move || if current_step.get() == total_steps - 1 {
                                            "Get Started 🚀"
                                        } else {
                                            "Next →"
                                        }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
