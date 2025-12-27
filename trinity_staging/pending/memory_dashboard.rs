**src/main.rs**
```rust
//! Trinity Memory Stats Dashboard – Sci‑Fi Terminal UI
//!
//! This crate builds a live dashboard widget using the Leptos framework.
//! It displays various memory‑related metrics of the Trinity system:
//!
//! 1️⃣ Fragment count (animated counter)  
//! 2️⃣ Embedding dimension visualisation (grid of squares)  
//! 3️⃣ Recent queries log (auto‑scrolling)  
//! 4️⃣ Similarity score distribution (mini bar chart)  
//! 5️⃣ Storage usage meter (horizontal gauge)  
//! 6️⃣ Auto‑refresh every 5 seconds  
//! 7️⃣ Sparkline charts for historical data  
//! 8️⃣ Expandable detail panels
//!
//! The UI is styled to look like a retro sci‑fi computer terminal.

use leptos::*;
use leptos::html::Div;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use gloo_timers::callback::Interval;

// -----------------------------------------------------------------------------
// Data structures – replace these with real API calls in production
// -----------------------------------------------------------------------------
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct DashboardData {
    fragment_count: u64,
    embedding_dim: usize,
    recent_queries: Vec<String>,
    similarity_distribution: Vec<u32>, // 10 buckets (0‑100%)
    storage_used_gb: f64,
    storage_total_gb: f64,
    sparkline_fragments: Vec<u64>,
    sparkline_storage: Vec<f64>,
}

// Mock async fetch – in a real app this would call an HTTP endpoint.
async fn fetch_dashboard_data() -> DashboardData {
    // Simulated random data
    use rand::Rng;
    let mut rng = rand::thread_rng();

    DashboardData {
        fragment_count: rng.gen_range(1_000..10_000),
        embedding_dim: 128,
        recent_queries: (0..5)
            .map(|i| format!("query_{:?} – {}", chrono::Utc::now(), i))
            .collect(),
        similarity_distribution: (0..10).map(|_| rng.gen_range(0..100)).collect(),
        storage_used_gb: rng.gen_range(20.0..80.0),
        storage_total_gb: 100.0,
        sparkline_fragments: (0..30)
            .map(|_| rng.gen_range(800..1200))
            .collect(),
        sparkline_storage: (0..30)
            .map(|_| rng.gen_range(20.0..80.0))
            .collect(),
    }
}

// -----------------------------------------------------------------------------
// Helper components
// -----------------------------------------------------------------------------
#[component]
fn AnimatedCounter(cx: Scope, value: u64) -> impl IntoView {
    // Simple animation using a signal that interpolates to the target.
    let displayed = create_rw_signal(cx, 0u64);
    let target = create_rw_signal(cx, value);

    // Update animation on target change
    create_effect(cx, move |_| {
        let tgt = target.get();
        let mut cur = displayed.get();

        if cur != tgt {
            // Linear step – tweak for smoother animation
            let step = ((tgt as i64 - cur as i64).abs() / 10).max(1) as u64;
            if cur < tgt {
                cur += step.min(tgt - cur);
            } else {
                cur -= step.min(cur - tgt);
            }
            displayed.set(cur);
        }
    });

    // Sync target when prop changes
    create_effect(cx, move |_| {
        target.set(value);
    });

    view! { cx,
        <span class="counter">{displayed}</span>
    }
}

#[component]
fn EmbeddingGrid(cx: Scope, dim: usize) -> impl IntoView {
    let size = (dim as f64).sqrt().ceil() as usize;
    view! { cx,
        <div class="embedding-grid">
            { (0..size*size).map(|i| view!{ cx,
                <div class=move || if i < dim {"grid-cell active"} else {"grid-cell"}></div>
            }).collect_view(cx) }
        </div>
    }
}

#[component]
fn RecentQueriesLog(cx: Scope, queries: Vec<String>) -> impl IntoView {
    view! { cx,
        <ul class="log-list">
            {queries.into_iter().map(|q| view!{cx,
                <li class="log-item">{q}</li>
            }).collect_view(cx)}
        </ul>
    }
}

#[component]
fn SimilarityBarChart(cx: Scope, buckets: Vec<u32>) -> impl IntoView {
    let max = *buckets.iter().max().unwrap_or(&1) as f64;
    view! { cx,
        <div class="bar-chart">
            {buckets.into_iter().map(move |v| {
                let height = (v as f64 / max) * 100.0;
                view!{cx,
                    <div class="bar" style=move || format!("height: {}%;", height)></div>
                }
            }).collect_view(cx)}
        </div>
    }
}

#[component]
fn StorageMeter(cx: Scope, used: f64, total: f64) -> impl IntoView {
    let percent = (used / total * 100.0).min(100.0);
    view! { cx,
        <div class="storage-meter">
            <div class="meter-fill" style=move || format!("width: {}%;", percent)></div>
            <span class="meter-label">{format!("{:.1}% ({:.1}/{:.1} GB)", percent, used, total)}</span>
        </div>
    }
}

#[component]
fn Sparkline(cx: Scope, data: Vec<f64>, color: &'static str) -> impl IntoView {
    // Simple SVG polyline sparkline
    let max = *data.iter().max_by(|a,b| a.partial_cmp(b).unwrap()).unwrap_or(&1.0);
    let min = *data.iter().min_by(|a,b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
    let points: String = data.iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f64;
            let y = if max - min == 0.0 {0.0} else {(max - v) / (max - min)}; // invert Y
            format!("{},{} ", x, y)
        })
        .collect();

    view!{ cx,
        <svg class="sparkline" viewBox=format!("0 0 {} 1", data.len()) preserveAspectRatio="none">
            <polyline points=points fill="none" stroke=color stroke-width="0.2"/>
        </svg>
    }
}

// -----------------------------------------------------------------------------
// Main dashboard component
// -----------------------------------------------------------------------------
#[component]
fn Dashboard(cx: Scope) -> impl IntoView {
    // Reactive state holding the latest data
    let data = create_rw_signal(cx, DashboardData::default());

    // Auto‑refresh every 5 seconds
    let _interval = Interval::new(5000, move || {
        spawn_local({
            let data = data.clone();
            async move {
                let fresh = fetch_dashboard_data().await;
                data.set(fresh);
            }
        });
    });

    // Initial load
    spawn_local({
        let data = data.clone();
        async move {
            data.set(fetch_dashboard_data().await);
        }
    });

    view! { cx,
        <div class="dashboard terminal">
            // 1️⃣ Fragment counter
            <section class="panel">
                <h2>Fragment Count</h2>
                <AnimatedCounter value=data.get().fragment_count />
            </section>

            // 2️⃣ Embedding dimension visualisation
            <section class="panel">
                <h2>Embedding Dimension ({data.get().embedding_dim})</h2>
                <EmbeddingGrid dim=data.get().embedding_dim />
            </section>

            // 3️⃣ Recent queries log (scrollable)
            <section class="panel scrollable">
                <h2>Recent Queries</h2>
                <RecentQueriesLog queries=data.get().recent_queries.clone() />
            </section>

            // 4️⃣ Similarity distribution mini bar chart
            <section class="panel">
                <h2>Similarity Distribution</h2>
                <SimilarityBarChart buckets=data.get().similarity_distribution.clone() />
            </section>

            // 5️⃣ Storage usage meter
            <section class="panel">
                <h2>Storage Usage</h2>
                <StorageMeter used=data.get().storage_used_gb total=data.get().storage_total_gb />
            </section>

            // 7️⃣ Sparkline historical data (fragments & storage)
            <section class="panel sparklines">
                <h2>Historical Fragments</h2>
                <Sparkline data=data.get().sparkline_fragments.iter().map(|v| *v as f64).collect() color="#0ff"/>
                <h2>Historical Storage (GB)</h2>
                <Sparkline data=data.get().sparkline_storage.clone() color="#f0f"/>
            </section>

            // 8️⃣ Expandable detail panels
            <details class="detail-panel">
                <summary>Advanced Details</summary>
                <pre class="json">{serde_json::to_string_pretty(&data.get()).unwrap()}</pre>
            </details>
        </div>
