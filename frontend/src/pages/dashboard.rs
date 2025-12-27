//! Trinity AI OS Dashboard
//!
//! Main landing page showing system status, quick actions, and real-time stats.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NotebookStats {
    pub source_count: usize,
    pub total_chunks: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    pub total_fragments: usize,
    pub conversations_stored: usize,
    pub facts_learned: usize,
}

#[component]
pub fn TrinityDashboard() -> impl IntoView {
    let (notebook_stats, set_notebook_stats) = signal(NotebookStats::default());
    let (memory_stats, set_memory_stats) = signal(MemoryStats::default());

    // Fetch stats on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/api/notebook/stats").send().await {
                if let Ok(data) = resp.json::<NotebookStats>().await {
                    set_notebook_stats.set(data);
                }
            }
        });

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/api/memory/stats").send().await {
                if let Ok(data) = resp.json::<MemoryStats>().await {
                    set_memory_stats.set(data);
                }
            }
        });
    });

    view! {
        <div class="min-h-screen p-8">
            // Hero Section
            <div class="max-w-6xl mx-auto mb-12">
                <div class="text-center mb-8">
                    <h1 class="text-5xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-amber-400 mb-4">
                        "Trinity AI OS"
                    </h1>
                    <p class="text-xl text-gray-400">
                        "Autonomous AI Agent Runtime • AMD Strix Halo Optimized"
                    </p>
                </div>

                // Status Indicator
                <div class="flex justify-center mb-8">
                    <div class="flex items-center gap-2 px-4 py-2 bg-green-500/20 border border-green-500/50 rounded-full">
                        <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                        <span class="text-green-400 text-sm font-medium">"System Online"</span>
                    </div>
                </div>
            </div>

            // Stats Cards
            <div class="max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
                // Notebook Stats Card
                <div class="bg-slate-800/50 backdrop-blur border border-white/10 rounded-xl p-6 hover:border-purple-500/50 transition-colors">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="p-3 bg-purple-500/20 rounded-lg">
                            <span class="text-2xl">"📚"</span>
                        </div>
                        <h3 class="text-lg font-semibold text-white">"Knowledge Base"</h3>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between">
                            <span class="text-gray-400">"Sources"</span>
                            <span class="text-white font-mono">{move || notebook_stats.get().source_count}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-400">"Chunks"</span>
                            <span class="text-white font-mono">{move || notebook_stats.get().total_chunks}</span>
                        </div>
                    </div>
                </div>

                // Memory Stats Card
                <div class="bg-slate-800/50 backdrop-blur border border-white/10 rounded-xl p-6 hover:border-blue-500/50 transition-colors">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="p-3 bg-blue-500/20 rounded-lg">
                            <span class="text-2xl">"🧠"</span>
                        </div>
                        <h3 class="text-lg font-semibold text-white">"Memory"</h3>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between">
                            <span class="text-gray-400">"Fragments"</span>
                            <span class="text-white font-mono">{move || memory_stats.get().total_fragments}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-400">"Conversations"</span>
                            <span class="text-white font-mono">{move || memory_stats.get().conversations_stored}</span>
                        </div>
                    </div>
                </div>

                // Agents Card
                <div class="bg-slate-800/50 backdrop-blur border border-white/10 rounded-xl p-6 hover:border-amber-500/50 transition-colors">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="p-3 bg-amber-500/20 rounded-lg">
                            <span class="text-2xl">"🤖"</span>
                        </div>
                        <h3 class="text-lg font-semibold text-white">"Agents"</h3>
                    </div>
                    <div class="space-y-2">
                        <div class="flex justify-between">
                            <span class="text-gray-400">"Active"</span>
                            <span class="text-green-400 font-mono">"3"</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-gray-400">"Queued Tasks"</span>
                            <span class="text-white font-mono">"0"</span>
                        </div>
                    </div>
                </div>
            </div>

            // Quick Actions
            <div class="max-w-6xl mx-auto">
                <h2 class="text-xl font-semibold text-white mb-4">"Quick Actions"</h2>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <a href="/notebook" class="group flex flex-col items-center p-6 bg-slate-800/30 border border-white/10 rounded-xl hover:bg-purple-500/20 hover:border-purple-500/50 transition-all">
                        <span class="text-3xl mb-2 group-hover:scale-110 transition-transform">"📝"</span>
                        <span class="text-gray-300 group-hover:text-white">"Add Source"</span>
                    </a>
                    <a href="/chat" class="group flex flex-col items-center p-6 bg-slate-800/30 border border-white/10 rounded-xl hover:bg-blue-500/20 hover:border-blue-500/50 transition-all">
                        <span class="text-3xl mb-2 group-hover:scale-110 transition-transform">"💬"</span>
                        <span class="text-gray-300 group-hover:text-white">"Chat"</span>
                    </a>
                    <a href="/agents" class="group flex flex-col items-center p-6 bg-slate-800/30 border border-white/10 rounded-xl hover:bg-amber-500/20 hover:border-amber-500/50 transition-all">
                        <span class="text-3xl mb-2 group-hover:scale-110 transition-transform">"🔀"</span>
                        <span class="text-gray-300 group-hover:text-white">"Workflows"</span>
                    </a>
                    <a href="/memory" class="group flex flex-col items-center p-6 bg-slate-800/30 border border-white/10 rounded-xl hover:bg-green-500/20 hover:border-green-500/50 transition-all">
                        <span class="text-3xl mb-2 group-hover:scale-110 transition-transform">"🔍"</span>
                        <span class="text-gray-300 group-hover:text-white">"Search Memory"</span>
                    </a>
                </div>
            </div>

            // Hardware Info
            <div class="max-w-6xl mx-auto mt-12">
                <div class="bg-gradient-to-r from-slate-800/50 to-slate-900/50 border border-white/10 rounded-xl p-6">
                    <h3 class="text-lg font-semibold text-white mb-3">"Hardware"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                        <div class="flex items-center gap-2">
                            <span class="text-blue-400">"CPU:"</span>
                            <span class="text-gray-300">"AMD Ryzen AI Max+ 395"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-green-400">"GPU:"</span>
                            <span class="text-gray-300">"RDNA 3.5 (40 CUs)"</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-purple-400">"NPU:"</span>
                            <span class="text-gray-300">"XDNA 2 (50 TOPS)"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
