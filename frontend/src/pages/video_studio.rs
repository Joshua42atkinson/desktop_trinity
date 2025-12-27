//! Video Studio - YouTube content creation workflow
//!
//! Script editor, thumbnail planning, and SEO tools.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Serialize)]
struct EnqueueRequest {
    name: String,
    description: String,
    priority: String,
    task_type: TaskTypeRequest,
}

#[derive(Serialize)]
struct TaskTypeRequest {
    #[serde(rename = "type")]
    task_type: String,
    prompt: Option<String>,
    language: Option<String>,
    output_path: Option<String>,
}

fn enqueue_ai_task(name: &str, prompt: &str) {
    let name = name.to_string();
    let prompt = prompt.to_string();
    spawn_local(async move {
        let request = EnqueueRequest {
            name: name.clone(),
            description: format!("AI assist for: {}", name),
            priority: "normal".to_string(),
            task_type: TaskTypeRequest {
                task_type: "generate_code".to_string(),
                prompt: Some(prompt),
                language: Some("markdown".to_string()),
                output_path: None,
            },
        };

        let _ = Request::post("/api/autonomous/enqueue")
            .header("Content-Type", "application/json")
            .json(&request)
            .unwrap()
            .send()
            .await;
    });
}

#[component]
pub fn VideoStudio() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("scripts");
    let (script_content, set_script_content) = signal(String::new());

    view! {
        <div class="min-h-screen p-8">
            // Header
            <div class="max-w-6xl mx-auto mb-8">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/studios" class="text-gray-400 hover:text-white transition-colors">
                        "← Studios"
                    </a>
                </div>
                <div class="flex items-center gap-4">
                    <span class="text-4xl">"🎬"</span>
                    <div>
                        <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-red-400 to-rose-500">
                            "Video Studio"
                        </h1>
                        <p class="text-gray-400">"YouTube content creation workflow"</p>
                    </div>
                </div>
            </div>

            // Tabs
            <div class="max-w-6xl mx-auto mb-6">
                <div class="flex gap-2 border-b border-white/10 pb-2">
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-red-500-20={move || active_tab.get() == "scripts"}
                        class:text-red-400={move || active_tab.get() == "scripts"}
                        class:text-gray-400={move || active_tab.get() != "scripts"}
                        on:click=move |_| set_active_tab.set("scripts")
                    >
                        "Scripts"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-red-500-20={move || active_tab.get() == "thumbnails"}
                        class:text-red-400={move || active_tab.get() == "thumbnails"}
                        class:text-gray-400={move || active_tab.get() != "thumbnails"}
                        on:click=move |_| set_active_tab.set("thumbnails")
                    >
                        "Thumbnails"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-red-500-20={move || active_tab.get() == "seo"}
                        class:text-red-400={move || active_tab.get() == "seo"}
                        class:text-gray-400={move || active_tab.get() != "seo"}
                        on:click=move |_| set_active_tab.set("seo")
                    >
                        "SEO"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-red-500-20={move || active_tab.get() == "calendar"}
                        class:text-red-400={move || active_tab.get() == "calendar"}
                        class:text-gray-400={move || active_tab.get() != "calendar"}
                        on:click=move |_| set_active_tab.set("calendar")
                    >
                        "Calendar"
                    </button>
                </div>
            </div>

            // Content Area
            <div class="max-w-6xl mx-auto">
                {move || match active_tab.get() {
                    "scripts" => view! {
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                                <div class="flex justify-between items-center mb-4">
                                    <h3 class="font-semibold text-white">"Projects"</h3>
                                    <button class="px-3 py-1 bg-red-500/20 text-red-400 rounded-lg text-sm hover:bg-red-500/30 transition-colors">
                                        "+ New"
                                    </button>
                                </div>
                                <div class="space-y-2">
                                    <div class="p-3 bg-slate-700/50 rounded-lg border border-white/5 cursor-pointer hover:border-red-500/50 transition-colors">
                                        <div class="font-medium text-white">"Getting Started Video"</div>
                                        <div class="text-xs text-gray-400">"Draft - 45% complete"</div>
                                    </div>
                                </div>
                            </div>
                            <div class="lg:col-span-2 bg-slate-800/50 border border-white/10 rounded-xl p-4">
                                <div class="flex justify-between items-center mb-4">
                                    <h3 class="font-semibold text-white">"Script Editor"</h3>
                                    <div class="flex gap-2">
                                        <button
                                            class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-lg text-sm hover:bg-purple-500/30 transition-colors"
                                            on:click={
                                                let content = script_content;
                                                move |_| enqueue_ai_task("Video Script Assist", &format!("Help improve this YouTube video script: {}", content.get()))
                                            }
                                        >
                                            "✨ AI Assist"
                                        </button>
                                        <button class="px-3 py-1 bg-green-500/20 text-green-400 rounded-lg text-sm hover:bg-green-500/30 transition-colors">
                                            "Save"
                                        </button>
                                    </div>
                                </div>
                                <textarea
                                    class="w-full h-96 bg-slate-900/50 border border-white/10 rounded-lg p-4 text-gray-300 font-mono text-sm resize-none focus:border-red-500/50 focus:outline-none"
                                    placeholder="Start writing your script..."
                                    prop:value={move || script_content.get()}
                                    on:input=move |ev| set_script_content.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                    }.into_any(),
                    "thumbnails" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <h3 class="font-semibold text-white mb-4">"Thumbnail Generator"</h3>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div>
                                    <label class="block text-sm text-gray-400 mb-2">"Title Text"</label>
                                    <input type="text" class="w-full bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white" placeholder="Your catchy title..." />
                                    <button class="mt-4 w-full px-4 py-3 bg-red-500 hover:bg-red-600 text-white rounded-lg font-semibold transition-colors">
                                        "Generate Concepts"
                                    </button>
                                </div>
                                <div class="bg-slate-900/50 border border-dashed border-white/20 rounded-lg aspect-video flex items-center justify-center">
                                    <span class="text-gray-500">"Thumbnail preview"</span>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "seo" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <h3 class="font-semibold text-white mb-4">"SEO Optimizer"</h3>
                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm text-gray-400 mb-2">"Video Title"</label>
                                    <input type="text" class="w-full bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white" placeholder="Your video title..." />
                                </div>
                                <div>
                                    <label class="block text-sm text-gray-400 mb-2">"Description"</label>
                                    <textarea class="w-full h-32 bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white resize-none" placeholder="Video description..."></textarea>
                                </div>
                                <button class="px-4 py-2 bg-purple-500/20 text-purple-400 rounded-lg hover:bg-purple-500/30 transition-colors">
                                    "AI Optimize"
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                    "calendar" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <h3 class="font-semibold text-white mb-4">"Content Calendar"</h3>
                            <div class="text-center py-12 text-gray-500">
                                "Calendar view coming soon..."
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! { <div></div> }.into_any()
                }}
            </div>
        </div>
    }
}
