use leptos::prelude::*;

/// Simple getting started modal - no localStorage complexity for now
#[component]
pub fn GettingStartedModal(
    /// Whether the modal is visible
    #[prop(into)]
    show: Signal<bool>,
    /// Callback to close the modal
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in">
                // Backdrop
                <div class="absolute inset-0 bg-black/70 backdrop-blur-sm"></div>

                // Modal
                <div class="relative z-10 max-w-4xl w-full max-h-[90vh] overflow-hidden rounded-2xl border border-white/20 bg-slate-900/95 backdrop-blur-xl shadow-2xl animate-slide-up">
                    // Header with gradient
                    <div class="relative overflow-hidden">
                        <div class="absolute inset-0 bg-gradient-to-r from-purple-600/20 to-cyan-600/20"></div>
                        <div class="absolute inset-0 bg-[url('/noise.svg')] opacity-10"></div>

                        <div class="relative p-8 border-b border-white/10">
                            <div class="flex items-start justify-between">
                                <div>
                                    <div class="inline-flex items-center px-3 py-1 rounded-full bg-cyan-500/20 border border-cyan-400/30 text-cyan-300 text-xs font-bold uppercase tracking-widest mb-4">
                                        <span class="w-1.5 h-1.5 bg-cyan-400 rounded-full mr-2 animate-pulse"></span>
                                        "Welcome"
                                    </div>
                                    <h1 class="text-4xl font-black text-white mb-2">
                                        "The Daydream " <span class="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-cyan-400">"Initiative"</span>
                                    </h1>
                                    <p class="text-lg text-slate-300 max-w-2xl">
                                        "A privacy-first platform for creating narrative-driven learning experiences through reflection and authoring."
                                    </p>
                                </div>
                                <button
                                    on:click=move |_| on_close.run(())
                                    class="p-2 rounded-lg hover:bg-white/10 transition-colors text-slate-400 hover:text-white"
                                    aria-label="Close modal"
                                >
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                    </svg>
                                </button>
                            </div>
                        </div>
                    </div>

                    // Content
                    <div class="p-8 overflow-y-auto max-h-[calc(90vh-15rem)]">
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                            // Sandbox
                            <div class="p-6 rounded-xl bg-gradient-to-br from-purple-900/30 to-purple-800/20 border border-purple-500/30 hover:border-purple-400/50 transition-all duration-200">
                                <div class="w-12 h-12 rounded-lg bg-purple-500/20 flex items-center justify-center mb-4">
                                    <svg class="w-6 h-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
                                    </svg>
                                </div>
                                <h3 class="text-xl font-bold text-white mb-2">"AI Mirror Sandbox"</h3>
                                <p class="text-slate-300 text-sm leading-relaxed mb-4">
                                    "Chat with a Socratic AI to reflect on your learning journey. The AI asks questions rather than giving answers."
                                </p>
                                <a href="/" class="inline-flex items-center text-purple-400 hover:text-purple-300 text-sm font-medium transition-colors">
                                    "Start Reflecting →"
                                </a>
                            </div>

                            // Authoring
                            <div class="p-6 rounded-xl bg-gradient-to-br from-cyan-900/30 to-cyan-800/20 border border-cyan-500/30 hover:border-cyan-400/50 transition-all duration-200">
                                <div class="w-12 h-12 rounded-lg bg-cyan-500/20 flex items-center justify-center mb-4">
                                    <svg class="w-6 h-6 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                                    </svg>
                                </div>
                                <h3 class="text-xl font-bold text-white mb-2">"Story Authoring"</h3>
                                <p class="text-slate-300 text-sm leading-relaxed mb-4">
                                    "Create interactive narrative graphs visually. No coding required — design branching stories with nodes and connections."
                                </p>
                                <a href="/authoring" class="inline-flex items-center text-cyan-400 hover:text-cyan-300 text-sm font-medium transition-colors">
                                    "Start Creating →"
                                </a>
                            </div>

                            // Research
                            <div class="p-6 rounded-xl bg-gradient-to-br from-green-900/30 to-green-800/20 border border-green-500/30 hover:border-green-400/50 transition-all duration-200">
                                <div class="w-12 h-12 rounded-lg bg-green-500/20 flex items-center justify-center mb-4">
                                    <svg class="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                                    </svg>
                                </div>
                                <h3 class="text-xl font-bold text-white mb-2">"Research Dashboard"</h3>
                                <p class="text-slate-300 text-sm leading-relaxed mb-4">
                                    "View your reflection history and learning analytics. Track your progress and insights over time."
                                </p>
                                <a href="/research" class="inline-flex items-center text-green-400 hover:text-green-300 text-sm font-medium transition-colors">
                                    "View Data →"
                                </a>
                            </div>
                        </div>

                        // Privacy & FERPA
                        <div class="p-6 rounded-xl bg-white/5 border border-white/10">
                            <div class="flex items-start gap-4">
                                <div class="flex-shrink-0">
                                    <div class="w-10 h-10 rounded-full bg-green-500/20 flex items-center justify-center">
                                        <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
                                        </svg>
                                    </div>
                                </div>
                                <div class="flex-1">
                                    <h4 class="text-lg font-bold text-white mb-2">"🔐 Privacy-First Architecture"</h4>
                                    <p class="text-slate-300 text-sm leading-relaxed">
                                        "All AI processing runs locally on your machine. Your reflections and data never leave your computer. FERPA/COPPA compliant by design."
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Footer
                    <div class="p-6 border-t border-white/10 bg-slate-900/80 flex items-center justify-end">
                        <button
                            on:click=move |_| on_close.run(())
                            class="px-6 py-2.5 bg-gradient-to-r from-purple-600 to-cyan-600 hover:from-purple-500 hover:to-cyan-500 rounded-lg text-white font-bold transition-all duration-200 shadow-lg shadow-purple-500/25"
                        >
                            "Let's Go! 🚀"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
