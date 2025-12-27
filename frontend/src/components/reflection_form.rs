#[cfg(feature = "ssr")]
use crate::server_functions::save_reflection;
#[cfg(feature = "ssr")]
use leptos::prelude::*;

#[cfg(feature = "ssr")]
#[component]
pub fn ReflectionForm() -> impl IntoView {
    let save_reflection_action = Action::new(|(reflection_text,): &(String,)| {
        let user_id = 100;
        let challenge_name = "Systems Thinking".to_string();
        let reflection_text_owned = reflection_text.clone();
        async move { save_reflection(user_id, challenge_name, reflection_text_owned).await }
    });

    let reflection_text = RwSignal::new(String::new());

    view! {
        <div class="bg-gray-800 bg-opacity-50 p-6 rounded-lg shadow-lg">
            <h2 class="text-2xl font-semibold text-pink-400 mb-4">Your Reflection</h2>
            <textarea
                class="w-full h-40 p-2 bg-gray-700 text-white rounded"
                on:input=move |ev| reflection_text.set(event_target_value(&ev))
                prop:value=move || reflection_text.get()
            ></textarea>
            <button
                class="mt-4 px-4 py-2 bg-teal-500 text-white rounded hover:bg-teal-600"
                on:click=move |_| {
                    save_reflection_action.dispatch((reflection_text.get(),));
                }
            >
                Save Reflection
            </button>
            <Show when=move || save_reflection_action.value().get().is_some()>
                {move || match save_reflection_action.value().get().unwrap() {
                    Ok(_) => view! { <p class="text-green-500">"Reflection Saved Successfully"</p> }.into_any(),
                    Err(e) => view! { <p class="text-red-500">{format!("Error Saving Reflection: {}", e)}</p> }.into_any(),
                }}
            </Show>
        </div>
    }
}
