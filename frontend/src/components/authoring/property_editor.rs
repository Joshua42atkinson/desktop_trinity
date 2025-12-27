use common::expert::StoryNode;
use leptos::prelude::*;

#[component]
pub fn PropertyEditor(
    #[prop(into)] node: RwSignal<StoryNode>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    let node_data = move || node.get();

    view! {
        <div class="absolute right-0 top-0 bottom-0 w-80 bg-slate-900 border-l border-white/10 p-4 shadow-xl z-20 flex flex-col gap-4">
            <div class="flex justify-between items-center border-b border-white/10 pb-2">
                <h2 class="text-lg font-bold text-white">"Properties"</h2>
                <button
                    class="text-slate-400 hover:text-white"
                    on:click=move |_| on_close.run(())
                >
                    "✕"
                </button>
            </div>

            <div class="space-y-2">
                <label class="block text-sm font-medium text-slate-400">"Title"</label>
                <input
                    type="text"
                    class="w-full bg-slate-800 border border-slate-700 rounded p-2 text-white focus:border-cyan-500 focus:outline-none"
                    prop:value=move || node_data().title
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        node.update(|n| n.title = val);
                    }
                />
            </div>

            <div class="space-y-2 flex-grow flex flex-col">
                <label class="block text-sm font-medium text-slate-400">"Content"</label>
                <textarea
                    class="w-full flex-grow bg-slate-800 border border-slate-700 rounded p-2 text-white focus:border-cyan-500 focus:outline-none font-mono text-sm resize-none"
                    prop:value=move || node_data().content
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        node.update(|n| n.content = val);
                    }
                />
            </div>

            <div class="text-xs text-slate-500">
                "ID: " {move || node_data().id}
            </div>

            <div class="pt-4 border-t border-white/10">
                <button
                    class="w-full px-4 py-2 bg-red-900/50 hover:bg-red-900 text-red-200 border border-red-800 rounded transition-colors text-sm font-bold"
                    on:click=move |_| on_delete.run(())
                >
                    "Delete Node"
                </button>
            </div>
        </div>
    }
}
