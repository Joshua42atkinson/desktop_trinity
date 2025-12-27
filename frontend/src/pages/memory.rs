//! Memory Page - Semantic memory search
//!
//! Search and view Trinity's long-term memory fragments.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryFragment {
    pub id: Uuid,
    pub content: String,
    pub source: String,
    pub relevance: f32,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    pub total_fragments: usize,
    pub conversations_stored: usize,
    pub facts_learned: usize,
}

#[component]
pub fn MemoryPage() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (memories, set_memories) = signal(Vec::<MemoryFragment>::new());
    let (loading, set_loading) = signal(false);
    let (stats, set_stats) = signal(MemoryStats::default());
    let (show_add_form, set_show_add_form) = signal(false);
    let (new_content, set_new_content) = signal(String::new());
    let (status_message, set_status_message) = signal(Option::<(String, bool)>::None);

    // Fetch stats on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/api/memory/stats").send().await {
                if let Ok(data) = resp.json::<MemoryStats>().await {
                    set_stats.set(data);
                }
            }
        });
    });

    let do_search = move || {
        let q = query.get();
        if q.is_empty() {
            return;
        }

        set_loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let encoded = js_sys::encode_uri_component(&q);
            let url = format!("/api/memory/recall?query={}&limit=10", encoded);

            match Request::get(&url).send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<Vec<MemoryFragment>>().await {
                        set_memories.set(data);
                    }
                }
                Err(e) => {
                    set_status_message.set(Some((format!("Search failed: {}", e), false)));
                }
            }
            set_loading.set(false);
        });
    };

    let seed_memories = move |_: web_sys::MouseEvent| {
        set_loading.set(true);
        set_status_message.set(Some(("Seeding example memories...".to_string(), true)));
        wasm_bindgen_futures::spawn_local(async move {
            match Request::post("/api/memory/seed").send().await {
                Ok(resp) if resp.ok() => {
                    if let Ok(data) = resp.json::<Vec<MemoryFragment>>().await {
                        set_status_message.set(Some((
                            format!("✓ Added {} example memories!", data.len()),
                            true,
                        )));
                        // Refresh stats
                        if let Ok(resp) = Request::get("/api/memory/stats").send().await {
                            if let Ok(s) = resp.json::<MemoryStats>().await {
                                set_stats.set(s);
                            }
                        }
                    }
                }
                Ok(resp) => {
                    set_status_message.set(Some((format!("Failed: {}", resp.status()), false)));
                }
                Err(e) => {
                    set_status_message.set(Some((format!("Error: {}", e), false)));
                }
            }
            set_loading.set(false);
        });
    };

    let add_memory = move |_: web_sys::MouseEvent| {
        let content = new_content.get();
        if content.is_empty() {
            return;
        }

        set_loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match Request::post("/api/memory/store")
                .json(&serde_json::json!({
                    "content": content,
                    "source": "user:manual"
                }))
                .expect("json")
                .send()
                .await
            {
                Ok(resp) if resp.ok() => {
                    set_status_message.set(Some(("✓ Memory stored!".to_string(), true)));
                    set_new_content.set(String::new());
                    set_show_add_form.set(false);
                    // Refresh stats
                    if let Ok(resp) = Request::get("/api/memory/stats").send().await {
                        if let Ok(s) = resp.json::<MemoryStats>().await {
                            set_stats.set(s);
                        }
                    }
                }
                _ => {
                    set_status_message.set(Some(("Failed to store memory".to_string(), false)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen p-8">
            <div class="max-w-4xl mx-auto">
                <div class="flex items-center justify-between mb-2">
                    <h1 class="text-4xl font-bold text-white">"🧠 Memory"</h1>
                    <div class="text-sm text-gray-400 bg-slate-800/50 px-3 py-1 rounded-full">
                        {move || format!("{} fragments", stats.get().total_fragments)}
                    </div>
                </div>
                <p class="text-gray-400 mb-8">"Search Trinity's semantic long-term memory"</p>

                // Status Message
                {move || status_message.get().map(|(msg, success)| {
                    let color = if success { "text-green-400 bg-green-500/10 border-green-500/30" } else { "text-red-400 bg-red-500/10 border-red-500/30" };
                    view! {
                        <div class=format!("mb-4 px-4 py-2 rounded-lg border {}", color)>
                            {msg}
                        </div>
                    }
                })}

                // Search Box
                <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4 mb-6">
                    <div class="flex gap-2">
                        <input
                            type="text"
                            placeholder="Search memories... (try 'Rust', 'GPU', 'code')"
                            class="flex-1 bg-slate-900/50 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
                            prop:value=move || query.get()
                            on:input=move |ev| set_query.set(event_target_value(&ev))
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" {
                                    do_search();
                                }
                            }
                        />
                        <button
                            class="bg-blue-600 hover:bg-blue-500 text-white font-medium px-6 py-3 rounded-lg transition-colors disabled:opacity-50"
                            on:click=move |_: web_sys::MouseEvent| do_search()
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "..." } else { "Search" }}
                        </button>
                    </div>
                </div>

                // Action Buttons
                <div class="flex gap-3 mb-6">
                    <button
                        class="flex items-center gap-2 bg-purple-600/20 hover:bg-purple-600/30 text-purple-400 border border-purple-500/30 px-4 py-2 rounded-lg transition-colors"
                        on:click=seed_memories
                        disabled=move || loading.get()
                    >
                        <span>"🌱"</span>
                        "Seed Example Memories"
                    </button>
                    <button
                        class="flex items-center gap-2 bg-green-600/20 hover:bg-green-600/30 text-green-400 border border-green-500/30 px-4 py-2 rounded-lg transition-colors"
                        on:click=move |_: web_sys::MouseEvent| set_show_add_form.update(|v| *v = !*v)
                    >
                        <span>"+"</span>
                        "Add Memory"
                    </button>
                </div>

                // Add Memory Form
                {move || if show_add_form.get() {
                    Some(view! {
                        <div class="bg-slate-800/50 border border-green-500/30 rounded-xl p-4 mb-6">
                            <h3 class="text-lg font-medium text-white mb-3">"Add New Memory"</h3>
                            <textarea
                                placeholder="Enter memory content..."
                                class="w-full bg-slate-900/50 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-green-500 resize-none"
                                rows="3"
                                prop:value=move || new_content.get()
                                on:input=move |ev| set_new_content.set(event_target_value(&ev))
                            ></textarea>
                            <div class="flex justify-end gap-2 mt-3">
                                <button
                                    class="text-gray-400 hover:text-white px-4 py-2 transition-colors"
                                    on:click=move |_: web_sys::MouseEvent| set_show_add_form.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="bg-green-600 hover:bg-green-500 text-white font-medium px-4 py-2 rounded-lg transition-colors disabled:opacity-50"
                                    on:click=add_memory
                                    disabled=move || new_content.get().is_empty() || loading.get()
                                >
                                    "Save Memory"
                                </button>
                            </div>
                        </div>
                    })
                } else {
                    None
                }}

                // Results
                <div class="space-y-4">
                    <For
                        each=move || memories.get()
                        key=|m| m.id
                        children=move |memory| {
                            let relevance_pct = (memory.relevance * 100.0) as i32;
                            let relevance_color = if memory.relevance > 0.7 {
                                "text-green-400"
                            } else if memory.relevance > 0.4 {
                                "text-yellow-400"
                            } else {
                                "text-gray-400"
                            };

                            view! {
                                <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4 hover:border-blue-500/50 transition-colors">
                                    <div class="flex justify-between items-start mb-2">
                                        <span class="text-xs text-gray-500 font-mono bg-slate-900/50 px-2 py-1 rounded">{memory.source}</span>
                                        <span class=format!("text-sm font-medium {}", relevance_color)>
                                            {format!("{}% match", relevance_pct)}
                                        </span>
                                    </div>
                                    <p class="text-gray-300">{memory.content}</p>
                                    <div class="mt-2 text-xs text-gray-600">{memory.created_at}</div>
                                </div>
                            }
                        }
                    />
                </div>

                // Empty State
                {move || if memories.get().is_empty() && !loading.get() {
                    Some(view! {
                        <div class="text-center py-16 text-gray-500">
                            <div class="text-6xl mb-4 opacity-30">"🔍"</div>
                            {if stats.get().total_fragments == 0 {
                                view! {
                                    <div>
                                        <p class="mb-4">"No memories stored yet."</p>
                                        <p class="text-sm">"Click "<strong class="text-purple-400">"Seed Example Memories"</strong>" to add test data,"</p>
                                        <p class="text-sm">"or add content via the Notebook."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <p>"Search for memories using keywords above"</p>
                                }.into_any()
                            }}
                        </div>
                    })
                } else {
                    None
                }}
            </div>
        </div>
    }
}
