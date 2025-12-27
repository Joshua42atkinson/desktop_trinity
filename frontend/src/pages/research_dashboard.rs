use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchEvent {
    pub timestamp: f64,
    pub event_type: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchLog {
    pub events: Vec<ResearchEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VirtueTopology {
    pub valor: f32,
    pub competence: f32,
    pub compassion: f32,
    pub self_efficacy: f32,
    pub self_esteem: f32,
    pub interdependence: f32,
}

#[component]
pub fn ResearchDashboard() -> impl IntoView {
    let (research_log, set_research_log) = signal(ResearchLog { events: vec![] });
    let (virtues, set_virtues) = signal(VirtueTopology::default());

    // Fetch data on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/research/log")
                .send()
                .await
            {
                if let Ok(log) = resp.json::<ResearchLog>().await {
                    set_research_log.set(log);
                }
            }

            if let Ok(resp) = gloo_net::http::Request::get("/api/research/virtues")
                .send()
                .await
            {
                if let Ok(v) = resp.json::<VirtueTopology>().await {
                    set_virtues.set(v);
                }
            }
        });
    });

    view! {
        <div class="p-8 bg-gray-900 text-white min-h-screen font-sans">
            <h1 class="text-3xl font-bold mb-6 text-purple-400">"Research Dashboard"</h1>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                // Virtue Topology Visualization
                <div class="bg-gray-800 p-6 rounded-lg shadow-lg border border-gray-700">
                    <h2 class="text-xl font-semibold mb-4 text-blue-300">"Virtue Topology"</h2>
                    <div class="space-y-4">
                        <VirtueBar label="Valor" value=move || virtues.get().valor color="bg-red-500" />
                        <VirtueBar label="Competence" value=move || virtues.get().competence color="bg-blue-500" />
                        <VirtueBar label="Compassion" value=move || virtues.get().compassion color="bg-green-500" />
                        <VirtueBar label="Self-Efficacy" value=move || virtues.get().self_efficacy color="bg-yellow-500" />
                        <VirtueBar label="Self-Esteem" value=move || virtues.get().self_esteem color="bg-purple-500" />
                        <VirtueBar label="Interdependence" value=move || virtues.get().interdependence color="bg-teal-500" />
                    </div>
                </div>

                // Research Log
                <div class="bg-gray-800 p-6 rounded-lg shadow-lg border border-gray-700 h-96 overflow-y-auto">
                    <h2 class="text-xl font-semibold mb-4 text-green-300">"Research Log"</h2>
                    <ul class="space-y-2">
                        <For
                            each=move || research_log.get().events
                            key=|event| event.timestamp.to_bits()
                            children=move |event| {
                                view! {
                                    <li class="bg-gray-700 p-3 rounded border-l-4 border-green-500">
                                        <div class="text-xs text-gray-400">{format!("{:.2}s", event.timestamp)}</div>
                                        <div class="font-bold text-sm text-white">{event.event_type}</div>
                                        <div class="text-sm text-gray-300 font-mono">{event.data}</div>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>
            </div>
        </div>
    }
}

#[component]
fn VirtueBar<F>(label: &'static str, value: F, color: &'static str) -> impl IntoView
where
    F: Fn() -> f32 + Clone + Send + Sync + 'static,
{
    let value_clone = value.clone();
    view! {
        <div>
            <div class="flex justify-between mb-1">
                <span class="text-sm font-medium text-gray-300">{label}</span>
                <span class="text-sm font-medium text-gray-300">{move || format!("{:.2}", value())}</span>
            </div>
            <div class="w-full bg-gray-700 rounded-full h-2.5">
                <div
                    class=format!("h-2.5 rounded-full {}", color)
                    style=move || format!("width: {}%", value_clone() * 100.0)
                ></div>
            </div>
        </div>
    }
}
