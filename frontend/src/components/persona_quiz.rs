use common::Dilemma;
use gloo_net::http::Request;
use leptos::prelude::*;

#[component]
pub fn PersonaQuiz() -> impl IntoView {
    let dilemmas = Resource::new(
        || (),
        |_| async move {
            match Request::get("/api/dilemmas").send().await {
                Ok(response) => response
                    .json::<Vec<Dilemma>>()
                    .await
                    .map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        },
    );

    view! {
        <div>
            <h1>"Persona Quiz"</h1>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || dilemmas.get().map(|dilemmas_result| {
                    match dilemmas_result {
                        Ok(dilemmas) => view! {
                            <ul class="space-y-6">
                                {dilemmas.into_iter().map(|dilemma| {
                                    view! {
                                        <li class="leptos-panel border border-gray-700 rounded-lg p-4">
                                            <h2 class="text-xl font-semibold">{dilemma.title}</h2>
                                            <p class="text-gray-300 mb-3">{dilemma.dilemma_text}</p>
                                            <ul class="space-y-2">
                                                {dilemma.choices.into_iter().map(|choice| {
                                                    view! {
                                                        <li>
                                                            <button
                                                                type="button"
                                                                class="w-full text-left p-3 rounded-md border border-gray-600 hover:bg-gray-700"
                                                            >
                                                                {choice.choice_text}
                                                            </button>
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any(),
                        Err(e) => view! { <p class="text-red-400">"Error: "{e}</p> }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}
