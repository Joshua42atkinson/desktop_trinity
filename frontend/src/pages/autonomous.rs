//! Autonomous Dashboard - Check-in interface for 24-hour operation
//!
//! Shows runtime status, pending/completed tasks, and activity log.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug, Deserialize)]
struct StatusResponse {
    is_running: bool,
    pending_tasks: usize,
    completed_tasks: usize,
    uptime_seconds: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
struct CheckpointResponse {
    generated_at: String,
    uptime_hours: f64,
    tasks_completed: usize,
    tasks_failed: usize,
    recent_activity: Vec<String>,
    recommendations: Vec<String>,
}

#[component]
pub fn AutonomousDashboard() -> impl IntoView {
    let (status, set_status) = signal(None::<StatusResponse>);
    let (checkpoint, set_checkpoint) = signal(None::<CheckpointResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Fetch status on mount
    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            // Fetch status
            match Request::get("/api/autonomous/status").send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<StatusResponse>().await {
                        set_status.set(Some(data));
                    }
                }
                Err(e) => set_error.set(Some(format!("Failed to fetch status: {}", e))),
            }

            // Fetch checkpoint
            match Request::get("/api/autonomous/checkpoint").send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<CheckpointResponse>().await {
                        set_checkpoint.set(Some(data));
                    }
                }
                Err(e) => set_error.set(Some(format!("Failed to fetch checkpoint: {}", e))),
            }

            set_loading.set(false);
        });
    });

    let refresh = move |_| {
        // Trigger refresh by toggling loading
        set_loading.set(true);
        spawn_local(async move {
            if let Ok(resp) = Request::get("/api/autonomous/status").send().await {
                if let Ok(data) = resp.json::<StatusResponse>().await {
                    set_status.set(Some(data));
                }
            }
            if let Ok(resp) = Request::get("/api/autonomous/checkpoint").send().await {
                if let Ok(data) = resp.json::<CheckpointResponse>().await {
                    set_checkpoint.set(Some(data));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen p-8">
            // Header
            <div class="max-w-6xl mx-auto mb-8">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-4">
                        <span class="text-4xl">"🤖"</span>
                        <div>
                            <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-purple-400 to-blue-500">
                                "Autonomous Runtime"
                            </h1>
                            <p class="text-gray-400">"24-hour operation dashboard"</p>
                        </div>
                    </div>
                    <button
                        class="px-4 py-2 bg-purple-500/20 text-purple-400 rounded-lg hover:bg-purple-500/30 transition-colors"
                        on:click=refresh
                    >
                        "Refresh"
                    </button>
                </div>
            </div>

            // Error display
            {move || error.get().map(|e| view! {
                <div class="max-w-6xl mx-auto mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-300">
                    {e}
                </div>
            })}

            // Status cards
            <div class="max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
                {move || status.get().map(|s| view! {
                    <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                        <div class="text-gray-400 text-sm mb-1">"Status"</div>
                        <div class="flex items-center gap-2">
                            <div class={if s.is_running { "w-3 h-3 bg-green-500 rounded-full animate-pulse" } else { "w-3 h-3 bg-red-500 rounded-full" }}></div>
                            <span class="text-xl font-bold text-white">
                                {if s.is_running { "Running" } else { "Stopped" }}
                            </span>
                        </div>
                    </div>
                    <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                        <div class="text-gray-400 text-sm mb-1">"Pending Tasks"</div>
                        <div class="text-2xl font-bold text-amber-400">{s.pending_tasks}</div>
                    </div>
                    <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                        <div class="text-gray-400 text-sm mb-1">"Completed"</div>
                        <div class="text-2xl font-bold text-green-400">{s.completed_tasks}</div>
                    </div>
                    <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                        <div class="text-gray-400 text-sm mb-1">"Uptime"</div>
                        <div class="text-2xl font-bold text-cyan-400">
                            {s.uptime_seconds.map(|secs| {
                                let hours = secs / 3600;
                                let mins = (secs % 3600) / 60;
                                format!("{}h {}m", hours, mins)
                            }).unwrap_or_else(|| "-".to_string())}
                        </div>
                    </div>
                }.into_any()).unwrap_or_else(|| view! {
                    <div class="col-span-4 text-center text-gray-500 py-8">
                        {if loading.get() { "Loading..." } else { "No status available" }}
                    </div>
                }.into_any())}
            </div>

            // Checkpoint Stats
            <div class="max-w-6xl mx-auto mb-8">
                 {move || checkpoint.get().map(|c| view! {
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                         <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4 flex items-center justify-between">
                            <div>
                                <div class="text-red-300 text-sm">"Tasks Failed"</div>
                                <div class="text-2xl font-bold text-red-400">{c.tasks_failed}</div>
                            </div>
                            <div class="text-3xl">"⚠️"</div>
                         </div>
                         <div class="bg-blue-500/10 border border-blue-500/20 rounded-xl p-4 flex items-center justify-between">
                            <div>
                                <div class="text-blue-300 text-sm">"Report Generated"</div>
                                <div class="text-sm font-mono text-blue-400">
                                    {c.generated_at.split('T').nth(1).unwrap_or("").split('.').next().unwrap_or("Just now").to_string()}
                                </div>
                            </div>
                            <div class="text-3xl">"🕒"</div>
                         </div>
                         <div class="bg-purple-500/10 border border-purple-500/20 rounded-xl p-4 flex items-center justify-between">
                             <div>
                                <div class="text-purple-300 text-sm">"Success Rate"</div>
                                <div class="text-2xl font-bold text-purple-400">
                                    {if c.tasks_completed + c.tasks_failed > 0 {
                                        format!("{:.1}%", (c.tasks_completed as f64 / (c.tasks_completed + c.tasks_failed) as f64) * 100.0)
                                    } else {
                                        "100%".to_string()
                                    }}
                                </div>
                             </div>
                             <div class="text-3xl">"📈"</div>
                         </div>
                    </div>
                    // Use unused fields to suppress warnings (conceptually used in these cards now)
                    <div class="hidden">
                        {c.uptime_hours} // Consumed via StatusResponse mostly, but available here
                    </div>
                 }.into_any())}
            </div>

            // Main content grid
            <div class="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Recent Activity
                <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                    <h3 class="font-semibold text-white mb-4">"Recent Activity"</h3>
                    {move || checkpoint.get().map(|c| {
                        if c.recent_activity.is_empty() {
                            view! {
                                <div class="text-gray-500 text-center py-8">
                                    "No activity yet. Trinity is waiting for tasks."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {c.recent_activity.into_iter().map(|activity| {
                                        let is_success = activity.starts_with("✓");
                                        view! {
                                            <div class={if is_success {
                                                "p-3 bg-green-500/10 border border-green-500/20 rounded-lg text-sm text-green-300"
                                            } else {
                                                "p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-300"
                                            }}>
                                                {activity}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }).unwrap_or_else(|| view! {
                        <div class="text-gray-500 text-center py-8">"Loading..."</div>
                    }.into_any())}
                </div>

                // Quick Actions
                <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                    <h3 class="font-semibold text-white mb-4">"Quick Actions"</h3>
                    <div class="space-y-3">
                        <button class="w-full px-4 py-3 bg-purple-500/20 text-purple-400 rounded-lg text-left hover:bg-purple-500/30 transition-colors">
                            "🔍 Scan Codebase for TODOs"
                        </button>
                        <button class="w-full px-4 py-3 bg-cyan-500/20 text-cyan-400 rounded-lg text-left hover:bg-cyan-500/30 transition-colors">
                            "🧠 Run Dream Cycle"
                        </button>
                        <button class="w-full px-4 py-3 bg-amber-500/20 text-amber-400 rounded-lg text-left hover:bg-amber-500/30 transition-colors">
                            "📝 Generate Checkpoint Report"
                        </button>
                        <a href="/studios" class="block w-full px-4 py-3 bg-rose-500/20 text-rose-400 rounded-lg text-left hover:bg-rose-500/30 transition-colors">
                            "🎨 Go to Creative Studios"
                        </a>
                    </div>
                </div>

                // Recommendations
                <div class="lg:col-span-2 bg-slate-800/50 border border-white/10 rounded-xl p-6">
                    <h3 class="font-semibold text-white mb-4">"Recommendations"</h3>
                    {move || checkpoint.get().map(|c| view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            {c.recommendations.into_iter().map(|rec| view! {
                                <div class="p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg text-blue-300">
                                    <span class="mr-2">"💡"</span>{rec}
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()).unwrap_or_else(|| view! {
                        <div class="text-gray-500">"No recommendations at this time."</div>
                    }.into_any())}
                </div>
            </div>
        </div>
    }
}
