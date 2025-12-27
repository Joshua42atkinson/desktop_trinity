//! Writing Studio - Novel and narrative authoring
//!
//! Bridges edutainment and education through narrative.

use gloo_net::http::Request;
use leptos::html;
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

#[derive(Serialize)]
struct AddSourceRequest {
    name: String,
    content: String,
}

fn add_source(name: String, content: String) {
    spawn_local(async move {
        let request = AddSourceRequest { name, content };
        let _ = Request::post("/api/notebook/sources")
            .header("Content-Type", "application/json")
            .json(&request)
            .unwrap()
            .send()
            .await;
    });
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
pub fn WritingStudio() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("projects");
    let (word_count, set_word_count) = signal(0usize);
    let (writing_content, set_writing_content) = signal(String::new());
    let (outline_prompt, set_outline_prompt) = signal(String::new());

    // Selection State
    let (show_toolbar, set_show_toolbar) = signal(false);
    let (selection_range, set_selection_range) = signal((0usize, 0usize));

    // Element ref for textarea
    let textarea_ref = NodeRef::<html::Textarea>::new();

    // Update word count when content changes
    Effect::new(move |_| {
        let content = writing_content.get();
        let count = content.split_whitespace().count();
        set_word_count.set(count);
    });

    let handle_mouseup = move |_| {
        if let Some(el) = textarea_ref.get() {
            let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
            let end = el.selection_end().ok().flatten().unwrap_or(0) as usize;

            if start != end {
                set_selection_range.set((start, end));
                set_show_toolbar.set(true);
            } else {
                set_show_toolbar.set(false);
            }
        }
    };

    let handle_keyup = move |_| {
        if let Some(el) = textarea_ref.get() {
            let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
            let end = el.selection_end().ok().flatten().unwrap_or(0) as usize;

            if start != end {
                set_selection_range.set((start, end));
                set_show_toolbar.set(true);
            } else {
                set_show_toolbar.set(false);
            }
        }
    };

    // AI Actions
    let action_rewrite = move |_| {
        let (start, end) = selection_range.get();
        let content = writing_content.get();
        if start < content.len() && end <= content.len() {
            let selected_text = &content[start..end];
            enqueue_ai_task(
                "Rewrite Text",
                &format!(
                    "Rewrite the following text to be more engaging and descriptive:\n\n{}",
                    selected_text
                ),
                "markdown",
            );
            set_show_toolbar.set(false);
        }
    };

    let action_describe = move |_| {
        let (start, end) = selection_range.get();
        let content = writing_content.get();
        if start < content.len() && end <= content.len() {
            let selected_text = &content[start..end];
            enqueue_ai_task(
                "Describe Detail",
                &format!("Describe the following noun or phrase with rich sensory details (sight, sound, smell, touch):\n\n{}", selected_text),
                "markdown",
            );
            set_show_toolbar.set(false);
        }
    };

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
                    <span class="text-4xl">"📖"</span>
                    <div>
                        <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-amber-400 to-orange-500">
                            "Writing Studio"
                        </h1>
                        <p class="text-gray-400">"Novels and edutainment narratives"</p>
                    </div>
                </div>
            </div>

            // Tabs
            <div class="max-w-6xl mx-auto mb-6">
                <div class="flex gap-2 border-b border-white/10 pb-2">
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-amber-500-20={move || active_tab.get() == "projects"}
                        class:text-amber-400={move || active_tab.get() == "projects"}
                        class:text-gray-400={move || active_tab.get() != "projects"}
                        on:click=move |_| set_active_tab.set("projects")
                    >
                        "Projects"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-amber-500-20={move || active_tab.get() == "write"}
                        class:text-amber-400={move || active_tab.get() == "write"}
                        class:text-gray-400={move || active_tab.get() != "write"}
                        on:click=move |_| set_active_tab.set("write")
                    >
                        "Write"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-amber-500-20={move || active_tab.get() == "outline"}
                        class:text-amber-400={move || active_tab.get() == "outline"}
                        class:text-gray-400={move || active_tab.get() != "outline"}
                        on:click=move |_| set_active_tab.set("outline")
                    >
                        "Outline"
                    </button>
                    <button
                        class="px-4 py-2 rounded-t-lg transition-colors"
                        class:bg-amber-500-20={move || active_tab.get() == "bible"}
                        class:text-amber-400={move || active_tab.get() == "bible"}
                        class:text-gray-400={move || active_tab.get() != "bible"}
                        on:click=move |_| set_active_tab.set("bible")
                    >
                        "Bible"
                    </button>
                </div>
            </div>

            // Content
            <div class="max-w-6xl mx-auto">
                {move || match active_tab.get() {
                    "projects" => view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            <div class="bg-slate-800/30 border border-dashed border-white/20 rounded-xl p-8 flex flex-col items-center justify-center cursor-pointer hover:border-amber-500/50 transition-colors">
                                <span class="text-4xl mb-2">"+"</span>
                                <span class="text-gray-400">"New Novel Project"</span>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6 hover:border-amber-500/50 transition-colors cursor-pointer">
                                <div class="flex items-start justify-between mb-4">
                                    <span class="text-3xl">"📕"</span>
                                    <span class="px-2 py-1 text-xs bg-amber-500/20 text-amber-400 rounded">"Novel"</span>
                                </div>
                                <h4 class="font-semibold text-white mb-1">"The Technomancer's Familiar"</h4>
                                <p class="text-sm text-gray-400 mb-4">"Fantasy edutainment series"</p>
                                <div class="space-y-2">
                                    <div class="flex justify-between text-sm">
                                        <span class="text-gray-500">"Progress"</span>
                                        <span class="text-white">"12 / 30 chapters"</span>
                                    </div>
                                    <div class="w-full h-2 bg-slate-700 rounded-full overflow-hidden">
                                        <div class="h-full bg-gradient-to-r from-amber-500 to-orange-500 rounded-full" style="width: 40%"></div>
                                    </div>
                                    <div class="flex justify-between text-sm">
                                        <span class="text-gray-500">"Words"</span>
                                        <span class="text-white">"45,230"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6 hover:border-amber-500/50 transition-colors cursor-pointer">
                                <div class="flex items-start justify-between mb-4">
                                    <span class="text-3xl">"📗"</span>
                                    <span class="px-2 py-1 text-xs bg-green-500/20 text-green-400 rounded">"Edutainment"</span>
                                </div>
                                <h4 class="font-semibold text-white mb-1">"Learning Adventures"</h4>
                                <p class="text-sm text-gray-400">"Educational narrative for games"</p>
                            </div>
                        </div>
                    }.into_any(),
                    "write" => view! {
                        <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4">
                                <h3 class="font-semibold text-white mb-4">"Chapters"</h3>
                                <div class="space-y-2">
                                    <div class="p-2 bg-amber-500/20 border border-amber-500/50 rounded-lg text-amber-400 text-sm cursor-pointer">
                                        "Ch 1: The Beginning"
                                    </div>
                                    <div class="p-2 bg-slate-700/50 rounded-lg text-gray-400 text-sm cursor-pointer hover:bg-slate-700 transition-colors">
                                        "Ch 2: Discovery"
                                    </div>
                                    <button class="w-full p-2 border border-dashed border-white/20 rounded-lg text-gray-500 text-sm hover:border-amber-500/50 transition-colors">
                                        "+ Add Chapter"
                                    </button>
                                </div>
                            </div>
                            <div class="lg:col-span-3">
                                <div class="bg-slate-800/50 border border-white/10 rounded-xl p-4 relative">
                                    <div class="flex justify-between items-center mb-4">
                                        <div>
                                            <h3 class="font-semibold text-white">"Chapter 1: The Beginning"</h3>
                                            <span class="text-sm text-gray-500">{move || format!("{} words", word_count.get())}</span>
                                        </div>
                                        <div class="flex gap-2">
                                            <button
                                                class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-lg text-sm hover:bg-purple-500/30 transition-colors flex items-center gap-2"
                                                on:click={
                                                    move |_| {
                                                        let content = writing_content.get();
                                                        enqueue_ai_task(
                                                            "Novel Continue",
                                                            &format!("Continue the following story chapter:\n\n{}", content),
                                                            "markdown"
                                                        )
                                                    }
                                                }
                                            >
                                                "✨ AI Continue"
                                            </button>
                                            <button class="px-3 py-1 bg-green-500/20 text-green-400 rounded-lg text-sm hover:bg-green-500/30 transition-colors">
                                                "Save"
                                            </button>
                                        </div>
                                    </div>

                                    // AI Toolbar (Contextual)
                                    <div class="absolute top-20 left-1/2 transform -translate-x-1/2 bg-slate-900 border border-amber-500/50 rounded-lg shadow-xl p-2 flex gap-2 z-10 transition-opacity duration-200"
                                         class:opacity-0={move || !show_toolbar.get()}
                                         class:pointer-events-none={move || !show_toolbar.get()}>
                                        <button class="px-3 py-1 text-sm text-white hover:bg-white/10 rounded flex items-center gap-2" on:click=action_rewrite>
                                            "🔄 Rewrite"
                                        </button>
                                        <button class="px-3 py-1 text-sm text-white hover:bg-white/10 rounded flex items-center gap-2" on:click=action_describe>
                                            "👁️ Describe"
                                        </button>
                                        <div class="w-px bg-white/20"></div>
                                        <button class="px-3 py-1 text-sm text-white hover:bg-white/10 rounded flex items-center gap-2">
                                            "Expand"
                                        </button>
                                    </div>

                                    <textarea
                                        node_ref=textarea_ref
                                        class="w-full h-[600px] bg-slate-900/50 border border-white/10 rounded-lg p-6 text-gray-300 font-serif text-lg leading-relaxed resize-none focus:border-amber-500/50 focus:outline-none"
                                        placeholder="Begin writing your chapter here..."
                                        prop:value={move || writing_content.get()}
                                        on:input=move |ev| set_writing_content.set(event_target_value(&ev))
                                        on:mouseup=handle_mouseup
                                        on:keyup=handle_keyup
                                    ></textarea>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "outline" => view! {
                        <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                            <div class="flex justify-between items-center mb-6">
                                <h3 class="font-semibold text-white">"Story Outline"</h3>
                                <div class="flex gap-2 items-center">
                                    <input
                                        type="text"
                                        class="bg-slate-900/50 border border-white/10 rounded-lg p-2 text-white text-sm w-64"
                                        placeholder="Outline prompt..."
                                        prop:value={move || outline_prompt.get()}
                                        on:input=move |ev| set_outline_prompt.set(event_target_value(&ev))
                                    />
                                    <a href="/agents" class="px-3 py-1 bg-cyan-500/20 text-cyan-400 rounded-lg text-sm hover:bg-cyan-500/30 transition-colors">
                                        "Node Canvas"
                                    </a>
                                    <button
                                        class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-lg text-sm hover:bg-purple-500/30 transition-colors"
                                        on:click={
                                            move |_| {
                                                enqueue_ai_task(
                                                    "Story Outline Gen",
                                                    &format!("Generate a story outline based on: {}", outline_prompt.get()),
                                                    "markdown"
                                                )
                                            }
                                        }
                                    >
                                        "AI Generate"
                                    </button>
                                </div>
                            </div>
                            <div class="space-y-4">
                                <div class="border-l-4 border-amber-500 pl-4">
                                    <h4 class="font-medium text-white mb-2">"Act 1: Setup"</h4>
                                    <div class="space-y-2 text-sm text-gray-400">
                                        <div class="p-2 bg-slate-900/50 rounded">"Introduce protagonist and world"</div>
                                        <div class="p-2 bg-slate-900/50 rounded">"Establish the ordinary world"</div>
                                        <div class="p-2 bg-slate-900/50 rounded">"Inciting incident"</div>
                                    </div>
                                </div>
                                <div class="border-l-4 border-orange-500 pl-4">
                                    <h4 class="font-medium text-white mb-2">"Act 2: Confrontation"</h4>
                                    <div class="space-y-2 text-sm text-gray-400">
                                        <div class="p-2 bg-slate-900/50 rounded">"Rising action and challenges"</div>
                                        <div class="p-2 bg-slate-900/50 rounded">"Midpoint revelation"</div>
                                    </div>
                                </div>
                                <div class="border-l-4 border-red-500 pl-4">
                                    <h4 class="font-medium text-white mb-2">"Act 3: Resolution"</h4>
                                    <div class="space-y-2 text-sm text-gray-400">
                                        <div class="p-2 bg-slate-900/50 rounded">"Climax"</div>
                                        <div class="p-2 bg-slate-900/50 rounded">"Resolution"</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    "bible" => view! {
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                                <div class="flex justify-between items-center mb-4">
                                    <h3 class="font-semibold text-white">"Characters"</h3>
                                    <button
                                        class="px-3 py-1 bg-amber-500/20 text-amber-400 rounded-lg text-sm hover:bg-amber-500/30 transition-colors"
                                        on:click={
                                            move |_| {
                                                if let Ok(Some(name)) = web_sys::window().unwrap().prompt_with_message("Enter Character Name:") {
                                                    if let Ok(Some(desc)) = web_sys::window().unwrap().prompt_with_message("Enter Description (Bio/Traits):") {
                                                        add_source(name, desc);
                                                    }
                                                }
                                            }
                                        }
                                    >
                                        "+ Add"
                                    </button>
                                </div>
                                <div class="space-y-3">
                                    <div class="p-3 bg-slate-900/50 rounded-lg border border-white/5 hover:border-amber-500/50 transition-colors cursor-pointer">
                                        <div class="flex items-center gap-3">
                                            <div class="w-10 h-10 bg-amber-500/20 rounded-full flex items-center justify-center text-amber-400">
                                                "P"
                                            </div>
                                            <div>
                                                <div class="font-medium text-white">"Protagonist"</div>
                                                <div class="text-xs text-gray-500">"Main character"</div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="bg-slate-800/50 border border-white/10 rounded-xl p-6">
                                <div class="flex justify-between items-center mb-4">
                                    <h3 class="font-semibold text-white">"World Building"</h3>
                                    <button
                                        class="px-3 py-1 bg-amber-500/20 text-amber-400 rounded-lg text-sm hover:bg-amber-500/30 transition-colors"
                                        on:click={
                                            move |_| {
                                                if let Ok(Some(name)) = web_sys::window().unwrap().prompt_with_message("Enter World Element Name:") {
                                                    if let Ok(Some(desc)) = web_sys::window().unwrap().prompt_with_message("Enter Description (Lore/Rules):") {
                                                        add_source(name, desc);
                                                    }
                                                }
                                            }
                                        }
                                    >
                                        "+ Add"
                                    </button>
                                </div>
                                <div class="space-y-3">
                                    <div class="p-3 bg-slate-900/50 rounded-lg border border-white/5 hover:border-amber-500/50 transition-colors cursor-pointer">
                                        <div class="font-medium text-white">"Setting"</div>
                                        <div class="text-sm text-gray-500">"Define your world"</div>
                                    </div>
                                    <div class="p-3 bg-slate-900/50 rounded-lg border border-white/5 hover:border-amber-500/50 transition-colors cursor-pointer">
                                        <div class="font-medium text-white">"Magic System"</div>
                                        <div class="text-sm text-gray-500">"Rules and limitations"</div>
                                    </div>
                                    <div class="p-3 bg-slate-900/50 rounded-lg border border-white/5 hover:border-amber-500/50 transition-colors cursor-pointer">
                                        <div class="font-medium text-white">"Lore"</div>
                                        <div class="text-sm text-gray-500">"History and backstory"</div>
                                    </div>
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
