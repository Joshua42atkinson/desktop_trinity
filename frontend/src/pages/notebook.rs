//! Notebook Page - RAG-powered knowledge interface
//!
//! Add sources, query with natural language, get grounded responses with citations.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: Uuid,
    pub name: String,
    pub chunk_count: usize,
    pub ingested_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citation {
    pub source_id: Uuid,
    pub text_snippet: String,
    pub relevance: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub answer: String,
    pub citations: Vec<Citation>,
}

#[component]
pub fn NotebookPage() -> impl IntoView {
    let (sources, set_sources) = signal(Vec::<Source>::new());
    let (query, set_query) = signal(String::new());
    let (response, set_response) = signal(Option::<QueryResponse>::None);
    let (loading, set_loading) = signal(false);
    let (new_source_name, set_new_source_name) = signal(String::new());
    let (new_source_content, set_new_source_content) = signal(String::new());

    // Fetch sources on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/api/notebook/sources").send().await {
                if let Ok(data) = resp.json::<Vec<Source>>().await {
                    set_sources.set(data);
                }
            }
        });
    });

    // Add source handler
    let add_source = move |_| {
        let name = new_source_name.get();
        let content = new_source_content.get();
        if name.is_empty() || content.is_empty() {
            return;
        }

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({
                "name": name,
                "content": content
            });

            if let Ok(resp) = Request::post("/api/notebook/sources")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await
            {
                if let Ok(source) = resp.json::<Source>().await {
                    set_sources.update(|s| s.push(source));
                    set_new_source_name.set(String::new());
                    set_new_source_content.set(String::new());
                }
            }
        });
    };

    // Query handler
    let submit_query = move |_| {
        let q = query.get();
        if q.is_empty() {
            return;
        }

        set_loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({ "query": q });

            if let Ok(resp) = Request::post("/api/notebook/query")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await
            {
                if let Ok(data) = resp.json::<QueryResponse>().await {
                    set_response.set(Some(data));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen p-8">
            <div class="max-w-6xl mx-auto">
                <h1 class="text-4xl font-bold text-white mb-2">"📚 Knowledge Notebook"</h1>
                <p class="text-gray-400 mb-8">"Add sources and query with RAG-powered search"</p>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    // Left: Sources
                    <div>
                        <h2 class="text-xl font-semibold text-white mb-4">"Sources"</h2>

                        // Add Source Form
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4 mb-4">
                            <input
                                type="text"
                                placeholder="Source name..."
                                class="w-full bg-slate-900/50 border border-white/20 rounded-lg px-4 py-2 text-white placeholder-gray-500 mb-2"
                                prop:value=move || new_source_name.get()
                                on:input=move |ev| set_new_source_name.set(event_target_value(&ev))
                            />
                            <textarea
                                placeholder="Paste content here..."
                                class="w-full bg-slate-900/50 border border-white/20 rounded-lg px-4 py-2 text-white placeholder-gray-500 h-32 resize-none mb-2"
                                prop:value=move || new_source_content.get()
                                on:input=move |ev| set_new_source_content.set(event_target_value(&ev))
                            ></textarea>
                            <button
                                class="w-full bg-purple-600 hover:bg-purple-500 text-white font-medium py-2 rounded-lg transition-colors"
                                on:click=add_source
                            >
                                "Add Source"
                            </button>
                        </div>

                        // Source List
                        <div class="space-y-2">
                            <For
                                each=move || sources.get()
                                key=|s| s.id
                                children=move |source| view! {
                                    <div class="bg-slate-800/30 border border-white/10 rounded-lg p-3 flex justify-between items-center">
                                        <div>
                                            <div class="text-white font-medium">{source.name}</div>
                                            <div class="text-gray-500 text-sm">{source.chunk_count}" chunks"</div>
                                        </div>
                                        <div class="text-purple-400 text-sm">"✓"</div>
                                    </div>
                                }
                            />
                        </div>
                    </div>

                    // Right: Query
                    <div>
                        <h2 class="text-xl font-semibold text-white mb-4">"Query"</h2>

                        // Query Input
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4 mb-4">
                            <textarea
                                placeholder="Ask a question about your sources..."
                                class="w-full bg-slate-900/50 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 h-24 resize-none mb-2"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                            ></textarea>
                            <button
                                class="w-full bg-blue-600 hover:bg-blue-500 text-white font-medium py-2 rounded-lg transition-colors disabled:opacity-50"
                                on:click=submit_query
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() { "Searching..." } else { "Search" }}
                            </button>
                        </div>

                        // Response
                        {move || response.get().map(|r| view! {
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                                <h3 class="text-lg font-semibold text-white mb-2">"Answer"</h3>
                                <p class="text-gray-300 whitespace-pre-wrap mb-4">{r.answer}</p>

                                <h4 class="text-sm font-semibold text-gray-400 mb-2">"Citations"</h4>
                                <div class="space-y-2">
                                    <For
                                        each=move || r.citations.clone()
                                        key=|c| c.source_id
                                        children=move |citation| view! {
                                            <div class="bg-slate-900/50 border-l-2 border-purple-500 pl-3 py-2">
                                                <div class="text-gray-300 text-sm">{citation.text_snippet}</div>
                                                <div class="text-purple-400 text-xs mt-1">
                                                    {format!("{:.0}% relevance", citation.relevance * 100.0)}
                                                </div>
                                            </div>
                                        }
                                    />
                                </div>
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>
    }
}
