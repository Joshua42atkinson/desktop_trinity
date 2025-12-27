use crate::ui_state::GlobalUiState;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = webkitSpeechRecognition)]
    #[derive(Clone, Debug)]
    pub type SpeechRecognition;

    #[wasm_bindgen(method, structural, js_class = "SpeechRecognition", js_name = start)]
    pub fn start(this: &SpeechRecognition);

    #[wasm_bindgen(method, structural, js_class = "SpeechRecognition", js_name = stop)]
    pub fn stop(this: &SpeechRecognition);

    #[wasm_bindgen(method, structural, js_class = "SpeechRecognition", js_name = abort)]
    pub fn abort(this: &SpeechRecognition);

    #[wasm_bindgen(method, setter, structural, js_class = "SpeechRecognition", js_name = continuous)]
    pub fn set_continuous(this: &SpeechRecognition, value: bool);

    #[wasm_bindgen(method, setter, structural, js_class = "SpeechRecognition", js_name = interimResults)]
    pub fn set_interim_results(this: &SpeechRecognition, value: bool);

    #[wasm_bindgen(method, setter, structural, js_class = "SpeechRecognition", js_name = lang)]
    pub fn set_lang(this: &SpeechRecognition, value: &str);

    #[wasm_bindgen(method, setter, structural, js_class = "SpeechRecognition", js_name = onresult)]
    pub fn set_onresult(this: &SpeechRecognition, value: Option<&js_sys::Function>);

    #[wasm_bindgen(method, setter, structural, js_class = "SpeechRecognition", js_name = onend)]
    pub fn set_onend(this: &SpeechRecognition, value: Option<&js_sys::Function>);

    #[wasm_bindgen(method, setter, structural, js_class = "SpeechRecognition", js_name = onerror)]
    pub fn set_onerror(this: &SpeechRecognition, value: Option<&js_sys::Function>);
}

// Wrapper to satisfy Send + Sync for Leptos signals (only safe in single-threaded WASM)
#[derive(Clone, Debug)]
struct SpeechRecognitionWrapper(SpeechRecognition);

unsafe impl Send for SpeechRecognitionWrapper {}
unsafe impl Sync for SpeechRecognitionWrapper {}

#[component]
pub fn VoiceControl() -> impl IntoView {
    let state = expect_context::<GlobalUiState>();
    let (recognition, set_recognition) = signal::<Option<SpeechRecognitionWrapper>>(None);

    // Initialize Speech Recognition
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let has_recognition =
                js_sys::Reflect::has(&window, &JsValue::from_str("webkitSpeechRecognition"))
                    .unwrap_or(false);

            if has_recognition {
                // Get constructor function
                let constructor_val =
                    js_sys::Reflect::get(&window, &JsValue::from_str("webkitSpeechRecognition"))
                        .unwrap();
                let constructor = constructor_val.unchecked_into::<js_sys::Function>();

                // Construct instance
                let r_js = js_sys::Reflect::construct(&constructor, &js_sys::Array::new()).unwrap();

                let r: SpeechRecognition = r_js.unchecked_into();

                r.set_continuous(true);
                r.set_interim_results(true);
                r.set_lang("en-US");

                set_recognition.set(Some(SpeechRecognitionWrapper(r)));
            } else {
                log::warn!("Speech recognition not supported in this browser.");
            }
        }
    });

    // Toggle Listening
    let toggle_listening = move |_| {
        let is_listening = state.is_listening.get();
        if let Some(wrapper) = recognition.get() {
            let r = &wrapper.0;

            if is_listening {
                r.stop();
                state.is_listening.set(false);
            } else {
                // Setup callbacks

                // On Result
                let on_result_cb = Closure::wrap(Box::new(move |e: JsValue| {
                    if let Ok(results) = js_sys::Reflect::get(&e, &JsValue::from_str("results")) {
                        let length = js_sys::Reflect::get(&results, &JsValue::from_str("length"))
                            .ok()
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as u32;

                        for i in 0..length {
                            if let Ok(result) =
                                js_sys::Reflect::get(&results, &JsValue::from_f64(i as f64))
                            {
                                let is_final =
                                    js_sys::Reflect::get(&result, &JsValue::from_str("isFinal"))
                                        .ok()
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);

                                if let Ok(alternatives) =
                                    js_sys::Reflect::get(&result, &JsValue::from_f64(0.0))
                                {
                                    let transcript = js_sys::Reflect::get(
                                        &alternatives,
                                        &JsValue::from_str("transcript"),
                                    )
                                    .ok()
                                    .and_then(|v| v.as_string())
                                    .unwrap_or_default();

                                    if is_final {
                                        state.last_transcript.set(transcript.clone());
                                        crate::utils::ai::enqueue_ai_task(
                                            crate::utils::ai::TaskTypeRequest::Research,
                                            transcript,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                r.set_onresult(Some(on_result_cb.as_ref().unchecked_ref()));
                on_result_cb.forget(); // Leak for demo

                // On End
                let on_end_cb = Closure::wrap(Box::new(move || {
                    state.is_listening.set(false);
                }) as Box<dyn FnMut()>);
                r.set_onend(Some(on_end_cb.as_ref().unchecked_ref()));
                on_end_cb.forget();

                r.start();
                state.is_listening.set(true);
            }
        }
    };

    view! {
        <button
            class=move || {
                let base = "p-3 rounded-full transition-all duration-300 backdrop-blur-md";
                if state.is_listening.get() {
                    format!("{} bg-red-500/80 hover:bg-red-600/80 text-white animate-pulse shadow-[0_0_15px_rgba(239,68,68,0.5)]", base)
                } else {
                    format!("{} bg-white/10 hover:bg-white/20 text-white/80 hover:text-white", base)
                }
            }
            on:click=toggle_listening
            title="Toggle Voice Control"
        >
            <Show
                when=move || state.is_listening.get()
                fallback=|| view! {
                    // Mic Icon
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
                        <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
                        <line x1="12" y1="19" x2="12" y2="23"></line>
                        <line x1="8" y1="23" x2="16" y2="23"></line>
                    </svg>
                }
            >
                // Stop/Wave Icon
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="1" y1="1" x2="23" y2="23"></line>
                    <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"></path>
                    <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"></path>
                    <line x1="12" y1="19" x2="12" y2="23"></line>
                    <line x1="8" y1="23" x2="16" y2="23"></line>
                </svg>
            </Show>
        </button>
    }
}
