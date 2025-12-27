//! Game Studio - Android and Roblox game development

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AvatarStatus {
    state: String,
    message: String,
}

#[component]
fn HoloEmitter() -> impl IntoView {
    let (status, set_status) = signal(AvatarStatus {
        state: "Idle".to_string(),
        message: "Initializing Holo-Link...".to_string(),
    });

    // Poll avatar status every 2 seconds
    spawn_local(async move {
        let mut interval = gloo_timers::future::IntervalStream::new(2000);
        while (futures::StreamExt::next(&mut interval).await).is_some() {
            if let Ok(resp) = Request::get("/api/game/avatar").send().await {
                if let Ok(new_status) = resp.json::<AvatarStatus>().await {
                    set_status.set(new_status);
                }
            }
        }
    });

    // Dynamic classes based on state
    let crystal_color = move || match status.get().state.as_str() {
        "Thinking" => "shadow-[0_0_30px_#a855f7] border-purple-500 bg-purple-500/20", // Purple
        "Coding" => "shadow-[0_0_30px_#22c55e] border-green-500 bg-green-500/20",     // Green
        _ => "shadow-[0_0_30px_#06b6d4] border-cyan-500 bg-cyan-500/20",              // Cyan (Idle)
    };

    let pulse_anim = move || match status.get().state.as_str() {
        "Thinking" => "animate-pulse duration-700",
        "Coding" => "animate-bounce duration-1000",
        _ => "animate-pulse duration-[3000ms]",
    };

    view! {
        <div class="fixed bottom-8 right-8 z-50 flex flex-col items-center gap-4">
            // Speech Bubble
            <div class="bg-slate-900/90 border border-white/10 rounded-xl p-4 max-w-xs backdrop-blur-md mb-2 transition-all duration-500 opacity-90 hover:opacity-100">
                <p class="text-sm text-gray-300 font-mono">
                    <span class="text-green-400 font-bold">"> "</span>
                    {move || status.get().message}
                </p>
            </div>

            // The Crystal (Avatar Representation)
            <div
                class=move || format!("w-16 h-16 rounded-full border-2 flex items-center justify-center backdrop-blur-xl transition-all duration-1000 {} {}", crystal_color(), pulse_anim())
            >
                <div class="w-8 h-8 bg-white/50 rounded-full blur-md"></div>
            </div>

            <span class="text-xs text-white/30 font-mono uppercase tracking-widest">{move || status.get().state}</span>
        </div>
    }
}

#[component]
pub fn GameStudio() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("android");
    let (gdd_title, set_gdd_title) = signal(String::new());
    let (gdd_genre, set_gdd_genre) = signal("Educational".to_string());
    let (gdd_concept, set_gdd_concept) = signal(String::new());

    view! {
        <div class="min-h-screen p-8 relative">
            // Inject Holo-Emitter
            <HoloEmitter />

            // Header
            <div class="max-w-6xl mx-auto mb-8">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/studios" class="text-gray-400 hover:text-white transition-colors">
                        "← Studios"
                    </a>
                </div>
                <div class="flex items-center gap-4">
                    <span class="text-4xl">"🎮"</span>
                    <div>
                        <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-green-400 to-emerald-500">
                            "Game Studio"
                        </h1>
                        <p class="text-gray-400">"Android and Roblox game development"</p>
                    </div>
                </div>
            </div>

            // Platform Tabs (Unchanged logic, just re-rendering to fit structure)
            <div class="max-w-6xl mx-auto mb-6">
                <div class="flex gap-2 border-b border-white/10 pb-2">
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors flex items-center gap-2"
                        class:bg-green-500-20={move || active_tab.get() == "android"}
                        class:text-green-400={move || active_tab.get() == "android"}
                        class:text-gray-400={move || active_tab.get() != "android"}
                        on:click=move |_| set_active_tab.set("android")
                    >
                        "Android"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors flex items-center gap-2"
                        class:bg-green-500-20={move || active_tab.get() == "roblox"}
                        class:text-green-400={move || active_tab.get() == "roblox"}
                        class:text-gray-400={move || active_tab.get() != "roblox"}
                        on:click=move |_| set_active_tab.set("roblox")
                    >
                        "Roblox"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors flex items-center gap-2"
                        class:bg-green-500-20={move || active_tab.get() == "gdd"}
                        class:text-green-400={move || active_tab.get() == "gdd"}
                        class:text-gray-400={move || active_tab.get() != "gdd"}
                        on:click=move |_| set_active_tab.set("gdd")
                    >
                        "GDD Editor"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors flex items-center gap-2"
                        class:bg-green-500-20={move || active_tab.get() == "assets"}
                        class:text-green-400={move || active_tab.get() == "assets"}
                        class:text-gray-400={move || active_tab.get() != "assets"}
                        on:click=move |_| set_active_tab.set("assets")
                    >
                        "Assets"
                    </button>
                </div>
            </div>

            // Content
            <div class="max-w-6xl mx-auto">
                {move || match active_tab.get() {
                    "android" => view! {
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                            <div class="lg:col-span-2 space-y-4">
                                <div class="flex justify-between items-center">
                                    <h3 class="font-semibold text-white">"Android Projects"</h3>
                                    <button class="px-3 py-1 bg-green-500/20 text-green-400 rounded-lg text-sm hover:bg-green-500/30 transition-colors">
                                        "+ New Project"
                                    </button>
                                </div>
                                <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4 hover:border-green-500/50 transition-colors cursor-pointer">
                                    <div class="flex justify-between items-start">
                                        <div>
                                            <h4 class="font-medium text-white">"Edutainment Adventure"</h4>
                                            <p class="text-sm text-gray-400">"Educational game with narrative elements"</p>
                                        </div>
                                        <span class="px-2 py-1 text-xs bg-yellow-500/20 text-yellow-400 rounded">"In Progress"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                                <h3 class="font-semibold text-white mb-4">"Quick Actions"</h3>
                                <div class="space-y-2">
                                    <button class="w-full px-4 py-3 bg-slate-700/50 text-left rounded-lg hover:bg-slate-700 transition-colors text-gray-300">
                                        "Generate GDD"
                                    </button>
                                    <button class="w-full px-4 py-3 bg-slate-700/50 text-left rounded-lg hover:bg-slate-700 transition-colors text-gray-300">
                                        "Asset Generator"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "roblox" => view! {
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                            <div class="lg:col-span-2 space-y-4">
                                <div class="flex justify-between items-center">
                                    <h3 class="font-semibold text-white">"Roblox Experiences"</h3>
                                    <button class="px-3 py-1 bg-green-500/20 text-green-400 rounded-lg text-sm hover:bg-green-500/30 transition-colors">
                                        "+ New Experience"
                                    </button>
                                </div>
                                <div class="bg-slate-800/30 border border-dashed border-white/10 rounded-xl p-8 text-center">
                                    <span class="text-gray-500">"Create your first Roblox experience"</span>
                                </div>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                                <h3 class="font-semibold text-white mb-4">"Roblox Tools"</h3>
                                <div class="space-y-2">
                                    <button class="w-full px-4 py-3 bg-slate-700/50 text-left rounded-lg hover:bg-slate-700 transition-colors text-gray-300">
                                        "Lua Script Generator"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "gdd" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <div class="flex justify-between items-center mb-6">
                                <h3 class="font-semibold text-white">"Game Design Document"</h3>
                                <button
                                    class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-lg text-sm hover:bg-purple-500/30 transition-colors"
                                    on:click={
                                        move |_| {
                                            enqueue_ai_task(
                                                &format!("GDD Gen: {}", gdd_title.get()),
                                                &format!("Generate a Game Design Document for a {} game titled '{}'. high concept: {}", gdd_genre.get(), gdd_title.get(), gdd_concept.get()),
                                                "markdown"
                                            )
                                        }
                                    }
                                >
                                    "AI Generate"
                                </button>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div>
                                    <label class="block text-sm text-gray-400 mb-2">"Game Title"</label>
                                    <input
                                        type="text"
                                        class="w-full bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white"
                                        placeholder="My Awesome Game"
                                        prop:value={move || gdd_title.get()}
                                        on:input=move |ev| set_gdd_title.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm text-gray-400 mb-2">"Genre"</label>
                                    <select
                                        class="w-full bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white"
                                        on:change=move |ev| set_gdd_genre.set(event_target_value(&ev))
                                    >
                                        <option value="Educational">"Educational"</option>
                                        <option value="Adventure">"Adventure"</option>
                                        <option value="Puzzle">"Puzzle"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="mt-4">
                                <label class="block text-sm text-gray-400 mb-2">"High Concept"</label>
                                <textarea
                                    class="w-full h-24 bg-slate-900/50 border border-white/10 rounded-lg p-3 text-white resize-none"
                                    placeholder="Describe your game..."
                                    prop:value={move || gdd_concept.get()}
                                    on:input=move |ev| set_gdd_concept.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                    }.into_any(),
                    "assets" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <h3 class="font-semibold text-white mb-4">"Asset Library"</h3>
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                <div class="aspect-square bg-slate-900/50 border border-dashed border-white/20 rounded-lg flex items-center justify-center cursor-pointer hover:border-green-500/50 transition-colors">
                                    <span class="text-3xl text-gray-500">"+"</span>
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
