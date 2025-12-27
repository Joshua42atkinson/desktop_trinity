use leptos::prelude::*;

#[derive(Clone)]
pub struct MessageSuggestion {
    pub category: String,
    pub text: String,
    pub icon: String,
}

#[component]
pub fn MessageSuggestions(
    /// Callback when a suggestion is clicked
    #[prop(into)]
    on_select: Callback<String>,
) -> impl IntoView {
    let suggestions = vec![
        MessageSuggestion {
            category: "Reflection".to_string(),
            text: "I'm thinking about how I approach learning new skills...".to_string(),
            icon: "💭".to_string(),
        },
        MessageSuggestion {
            category: "Learning Journey".to_string(),
            text: "What patterns do I notice in how I tackle challenges?".to_string(),
            icon: "🎯".to_string(),
        },
        MessageSuggestion {
            category: "Obstacles".to_string(),
            text: "I'm feeling stuck on understanding this concept...".to_string(),
            icon: "🧗".to_string(),
        },
        MessageSuggestion {
            category: "Goals".to_string(),
            text: "I want to develop better habits for deep work.".to_string(),
            icon: "🚀".to_string(),
        },
    ];

    view! {
        <div class="space-y-3">
            <div class="flex items-center gap-2 mb-4">
                <div class="w-1 h-4 bg-gradient-to-b from-purple-500 to-cyan-500 rounded-full"></div>
                <p class="text-sm font-semibold text-slate-400 uppercase tracking-wider">
                    "Try asking about..."
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {suggestions.into_iter().map(|suggestion| {
                    let text = suggestion.text.clone();
                    view! {
                        <button
                            on:click=move |_| on_select.run(text.clone())
                            class="group p-4 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 hover:border-purple-500/50 transition-all duration-200 text-left"
                        >
                            <div class="flex items-start gap-3">
                                <div class="text-2xl flex-shrink-0 group-hover:scale-110 transition-transform">
                                    {suggestion.icon}
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div class="text-xs font-semibold text-purple-400 mb-1">
                                        {suggestion.category}
                                    </div>
                                    <div class="text-sm text-slate-300 group-hover:text-white transition-colors">
                                        {suggestion.text}
                                    </div>
                                </div>
                            </div>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
