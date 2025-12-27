//! Chat Page - Conversation interface with Trinity
//!
//! Real-time chat with the AI agent.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelStatusResponse {
    pub loaded_models: Vec<String>,
    pub total_vram_gb: f32,
    pub used_vram_gb: f32,
    pub active_tier: Option<String>,
}

#[component]
pub fn ChatPage() -> impl IntoView {
    let (messages, set_messages) = signal(vec![
        Message {
            role: "assistant".to_string(),
            content: "Hello! I'm Trinity, your AI assistant. I can help you with research, code, and creative tasks. What would you like to work on?".to_string(),
        }
    ]);
    let (input, set_input) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (model_status, set_model_status) = signal(ModelStatusResponse::default());

    // Poll model status every 2 seconds
    Effect::new(move |_| {
        let handle = set_interval(
            move || {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = gloo_net::http::Request::get("/api/models/status")
                        .send()
                        .await
                    {
                        if let Ok(status) = res.json::<ModelStatusResponse>().await {
                            set_model_status.set(status);
                        }
                    }
                });
            },
            std::time::Duration::from_secs(2),
        );
        on_cleanup(move || {
            if let Ok(h) = handle {
                h.clear();
            }
        });
    });

    let send_message = move || {
        let msg = input.get();
        if msg.is_empty() {
            return;
        }

        // Add user message
        set_messages.update(|m| {
            m.push(Message {
                role: "user".to_string(),
                content: msg.clone(),
            })
        });
        set_input.set(String::new());
        set_loading.set(true);

        // Connect to actual LLM endpoint
        wasm_bindgen_futures::spawn_local(async move {
            let res = gloo_net::http::Request::post("/api/chat")
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({ "message": msg }))
                .expect("Failed to create request")
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.ok() {
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if let Some(reply) = json.get("response").and_then(|v| v.as_str()) {
                                set_messages.update(|m| {
                                    m.push(Message {
                                        role: "assistant".to_string(),
                                        content: reply.to_string(),
                                    })
                                });
                            } else {
                                set_messages.update(|m| {
                                    m.push(Message {
                                        role: "assistant".to_string(),
                                        content: "Error: Invalid response format from server."
                                            .to_string(),
                                    })
                                });
                            }
                        } else {
                            set_messages.update(|m| {
                                m.push(Message {
                                    role: "assistant".to_string(),
                                    content: "Error: Could not parse server response.".to_string(),
                                })
                            });
                        }
                    } else {
                        set_messages.update(|m| {
                            m.push(Message {
                                role: "assistant".to_string(),
                                content: format!("Server Error: {}", response.status()),
                            })
                        });
                    }
                }
                Err(e) => {
                    set_messages.update(|m| {
                        m.push(Message {
                            role: "assistant".to_string(),
                            content: format!("Error connecting to Trinity backend: {}", e),
                        })
                    });
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="h-[calc(100vh-4rem)] flex overflow-hidden bg-slate-900 text-white font-inter">
            // Sidebar - Model Info (LM Studio Style)
            <div class="w-80 bg-slate-950 border-r border-white/10 flex flex-col p-4 space-y-6">
                <div class="space-y-2">
                    <h2 class="text-xs font-bold text-gray-500 uppercase tracking-wider">"Active Model"</h2>
                    <div class="p-3 bg-slate-900 rounded-lg border border-purple-500/30 flex items-center gap-3">
                        <div class="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]"></div>
                        <div>
                            <div class="font-bold text-sm text-purple-200 truncate w-48">
                                {move || model_status.get().loaded_models.first().cloned().unwrap_or("No Model Loaded".to_string())}
                            </div>
                            <div class="text-xs text-purple-400/60">"Ready for Inference"</div>
                        </div>
                    </div>
                </div>

                <div class="space-y-2">
                     <h2 class="text-xs font-bold text-gray-500 uppercase tracking-wider">"Resource Usage"</h2>
                     <div class="bg-slate-900 rounded-lg p-3 border border-white/5 space-y-3">
                        <div>
                            <div class="flex justify-between text-xs mb-1">
                                <span class="text-gray-400">"VRAM"</span>
                                <span class="text-gray-300">
                                    {move || format!("{:.1} / {:.1} GB", model_status.get().used_vram_gb, model_status.get().total_vram_gb)}
                                </span>
                            </div>
                            <div class="w-full bg-slate-800 rounded-full h-1.5">
                                <div
                                    class="bg-cyan-500 h-1.5 rounded-full transition-all duration-500"
                                    style=move || format!("width: {}%", (model_status.get().used_vram_gb / model_status.get().total_vram_gb.max(1.0)) * 100.0)
                                ></div>
                            </div>
                        </div>

                        <div class="flex justify-between items-center text-xs pt-2 border-t border-white/5">
                            <span class="text-gray-400">"Provider"</span>
                            <span class="px-2 py-0.5 rounded bg-white/10 text-white">"ROCm/HIP"</span>
                        </div>
                     </div>
                </div>

                <div class="space-y-2">
                    <h2 class="text-xs font-bold text-gray-500 uppercase tracking-wider">"Context"</h2>
                    <div class="text-xs text-gray-400 bg-slate-900 p-3 rounded-lg border border-white/5">
                        <p>"System Prompt: Enabled"</p>
                        <p class="mt-1">"Memories: Access Granted"</p>
                        <p class="mt-1">"Tools: Auto-Detect"</p>
                    </div>
                </div>

                <div class="mt-auto pt-4 border-t border-white/10">
                    <div class="text-[10px] text-gray-600 text-center">
                        "Trinity AI OS v0.1.0"
                    </div>
                </div>
            </div>

            // Main Chat Area
            <div class="flex-1 flex flex-col min-w-0 bg-slate-900">
                // Messages Scroll Area
                <div class="flex-1 overflow-y-auto p-4 space-y-6 scroll-smooth">
                    <For
                        each=move || messages.get()
                        key=|m| format!("{}{}", m.role, m.content.len())
                        children=move |message| {
                            let is_user = message.role == "user";
                            let align = if is_user { "justify-end" } else { "justify-start" };
                            let bubble_bg = if is_user { "bg-blue-600 text-white" } else { "bg-slate-800 text-gray-100" };
                            let rounded = if is_user { "rounded-2xl rounded-tr-sm" } else { "rounded-2xl rounded-tl-sm" };

                            view! {
                                <div class=format!("flex {}", align)>
                                    <div class=format!("max-w-[75%] {} shadow-lg {} p-5 leading-relaxed selection:bg-purple-500/30", bubble_bg, rounded)>
                                        <p class="whitespace-pre-wrap font-sans text-sm">{message.content}</p>
                                    </div>
                                </div>
                            }
                        }
                    />

                    {move || if loading.get() {
                        Some(view! {
                            <div class="flex justify-start">
                                <div class="bg-slate-800 rounded-2xl rounded-tl-sm p-4 shadow-lg flex items-center gap-2">
                                    <div class="text-xs text-gray-400 font-mono animate-pulse">"Thinking..."</div>
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }}
                </div>

                // Input Area
                <div class="p-4 bg-slate-900 border-t border-white/10">
                    <div class="relative max-w-4xl mx-auto">
                        <input
                            type="text"
                            placeholder="Send a message..."
                            class="w-full bg-slate-800 border-2 border-slate-700/50 rounded-xl px-5 py-4 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors shadow-inner"
                            prop:value=move || input.get()
                            on:input=move |ev| set_input.set(event_target_value(&ev))
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" && !ev.shift_key() {
                                    ev.prevent_default();
                                    send_message();
                                }
                            }
                        />
                        <button
                            class="absolute right-2 top-2 bottom-2 bg-blue-600 hover:bg-blue-500 text-white px-4 rounded-lg transition-all disabled:opacity-0 disabled:translate-x-2"
                            on:click=move |_: web_sys::MouseEvent| send_message()
                            disabled=move || loading.get() || input.get().is_empty()
                        >
                            "Send"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
