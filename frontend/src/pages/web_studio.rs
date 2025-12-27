//! Web Studio - Website building and deployment

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

fn enqueue_ai_task(name: &str, prompt: &str, language: &str) {
    let name = name.to_string();
    let prompt = prompt.to_string();
    let language = language.to_string();
    spawn_local(async move {
        let request = EnqueueRequest {
            name: name.clone(),
            description: format!("AI assist for: {}", name),
            priority: "normal".to_string(),
            task_type: TaskTypeRequest {
                task_type: "generate_code".to_string(),
                prompt: Some(prompt),
                language: Some(language),
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
pub fn WebStudio() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("sites");
    let (component_prompt, set_component_prompt) = signal(String::new());

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
                    <span class="text-4xl">"🌐"</span>
                    <div>
                        <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 to-blue-500">
                            "Web Studio"
                        </h1>
                        <p class="text-gray-400">"Website building and deployment"</p>
                    </div>
                </div>
            </div>

            // Tabs
            <div class="max-w-6xl mx-auto mb-6">
                <div class="flex gap-2 border-b border-white/10 pb-2">
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-cyan-500-20={move || active_tab.get() == "sites"}
                        class:text-cyan-400={move || active_tab.get() == "sites"}
                        class:text-gray-400={move || active_tab.get() != "sites"}
                        on:click=move |_| set_active_tab.set("sites")
                    >
                        "Sites"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-cyan-500-20={move || active_tab.get() == "components"}
                        class:text-cyan-400={move || active_tab.get() == "components"}
                        class:text-gray-400={move || active_tab.get() != "components"}
                        on:click=move |_| set_active_tab.set("components")
                    >
                        "Components"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-cyan-500-20={move || active_tab.get() == "deploy"}
                        class:text-cyan-400={move || active_tab.get() == "deploy"}
                        class:text-gray-400={move || active_tab.get() != "deploy"}
                        on:click=move |_| set_active_tab.set("deploy")
                    >
                        "Deploy"
                    </button>
                </div>
            </div>

            // Content
            <div class="max-w-6xl mx-auto">
                {move || match active_tab.get() {
                    "sites" => view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            <div class="bg-slate-800/30 border border-dashed border-white/20 rounded-xl p-8 flex flex-col items-center justify-center cursor-pointer hover:border-cyan-500/50 transition-colors">
                                <span class="text-4xl mb-2">"+"</span>
                                <span class="text-gray-400">"Create New Site"</span>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl overflow-hidden hover:border-cyan-500/50 transition-colors">
                                <div class="h-32 bg-gradient-to-br from-cyan-900/50 to-blue-900/50 flex items-center justify-center">
                                    <span class="text-5xl opacity-50">"🌐"</span>
                                </div>
                                <div class="p-4">
                                    <h4 class="font-medium text-white">"Portfolio Site"</h4>
                                    <p class="text-sm text-gray-400">"Personal portfolio template"</p>
                                    <div class="mt-3 flex gap-2">
                                        <button class="px-3 py-1 bg-cyan-500/20 text-cyan-400 rounded text-xs hover:bg-cyan-500/30 transition-colors">
                                            "Edit"
                                        </button>
                                        <button class="px-3 py-1 bg-slate-700/50 text-gray-400 rounded text-xs hover:bg-slate-700 transition-colors">
                                            "Preview"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "components" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <div class="flex flex-col gap-4 mb-6">
                                <div class="flex justify-between items-center">
                                    <h3 class="font-semibold text-white">"Component Library"</h3>
                                    <button
                                        class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-lg text-sm hover:bg-purple-500/30 transition-colors"
                                        on:click={
                                            move |_| {
                                                enqueue_ai_task(
                                                    "Web Component Gen",
                                                    &format!("Generate a web component: {}", component_prompt.get()),
                                                    "javascript"
                                                )
                                            }
                                        }
                                    >
                                        "AI Generate"
                                    </button>
                                </div>
                                <input
                                    type="text"
                                    class="w-full bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white"
                                    placeholder="Describe component to generate (e.g. 'Modern pricing card with toggle')"
                                    prop:value={move || component_prompt.get()}
                                    on:input=move |ev| set_component_prompt.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                <div class="p-4 bg-slate-900/50 border border-white/10 rounded-lg text-center hover:border-cyan-500/50 transition-colors cursor-pointer">
                                    <span class="text-sm text-gray-400">"Hero Section"</span>
                                </div>
                                <div class="p-4 bg-slate-900/50 border border-white/10 rounded-lg text-center hover:border-cyan-500/50 transition-colors cursor-pointer">
                                    <span class="text-sm text-gray-400">"Feature Grid"</span>
                                </div>
                                <div class="p-4 bg-slate-900/50 border border-white/10 rounded-lg text-center hover:border-cyan-500/50 transition-colors cursor-pointer">
                                    <span class="text-sm text-gray-400">"Contact Form"</span>
                                </div>
                                <div class="p-4 bg-slate-900/50 border border-white/10 rounded-lg text-center hover:border-cyan-500/50 transition-colors cursor-pointer">
                                    <span class="text-sm text-gray-400">"Navigation"</span>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "deploy" => view! {
                        <div class="space-y-6">
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                                <h3 class="font-semibold text-white mb-4">"Deployment Status"</h3>
                                <div class="space-y-3">
                                    <div class="flex items-center justify-between p-3 bg-slate-900/50 rounded-lg">
                                        <div class="flex items-center gap-3">
                                            <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                                            <span class="text-white">"portfolio.trinity.dev"</span>
                                        </div>
                                        <span class="text-sm text-gray-400">"Deployed 2h ago"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                                <h3 class="font-semibold text-white mb-4">"Deploy New Site"</h3>
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm text-gray-400 mb-2">"Domain"</label>
                                        <input type="text" class="w-full bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white" placeholder="mysite.com" />
                                    </div>
                                    <button class="w-full px-4 py-3 bg-cyan-500 hover:bg-cyan-600 text-white rounded-lg font-semibold transition-colors">
                                        "Deploy"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! { <div></div> }.into_any()
                }}
            </div>
        </div>
    }
}
