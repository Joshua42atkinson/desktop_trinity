use crate::ui_state::GlobalUiState;
use leptos::prelude::*;

#[component]
pub fn ZenToggle() -> impl IntoView {
    let state = use_context::<GlobalUiState>().expect("GlobalUiState missing");
    let (zen, _set_zen) = (state.zen_mode, state.zen_mode);

    view! {
        <div class="fixed bottom-4 right-4 z-50 flex flex-col items-center gap-4">
            <Show when=move || zen.get()>
                <crate::components::voice::voice_control::VoiceControl />
            </Show>

            <button
                class="p-2 rounded-full transition-all duration-300 shadow-lg hover:scale-110"
                class:bg-cyan-500={move || !zen.get()}
                class:bg-purple-600={move || zen.get()}
                on:click=move |_| state.toggle_zen()
                title={move || if zen.get() { "Exit Zen Mode" } else { "Enter Zen Mode" }}
            >
                {move || if zen.get() { "⭕" } else { "🧘" }}
            </button>
        </div>
    }
}
