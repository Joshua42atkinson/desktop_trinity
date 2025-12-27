use leptos::*;
use crate::components::glass_panel::GlassPanel;
use crate::models::Artifact;
use crate::components::artifact_card::ArtifactCard;

#[component]
pub fn Daydream() -> impl IntoView {
    // Define the artifacts for this page
    let artifacts = vec![
        Artifact {
            title: "Project Synthesis".to_string(),
            description: "A comprehensive breakdown of the Pedagogy, Gamification, and Technical Architecture.".to_string(),
            tags: vec!["Research".to_string(), "Architecture".to_string()],
            link: Some("https://docs.google.com/document/d/1EtW32Etg58ZEyc-8R_fQUwyum-7cEoC3qnCr7rn5wkQ/edit?usp=sharing".to_string()),
            link_text: "Read Strategy".to_string(),
            icon: "<svg fill='none' stroke='currentColor' viewBox='0 0 24 24'><path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z'></path></svg>".to_string(),
        },
        Artifact {
            title: "VaaM Research Report".to_string(),
            description: "The 'Vocabulary-as-a-Mechanic' Model: Moving beyond rote memorization using Situated Cognition.".to_string(),
            tags: vec!["Learning Science".to_string(), "Gamification".to_string()],
            link: Some("https://docs.google.com/document/d/1Nlm2Q5MFzGaa3uL6Xry6gCMrtDjIKw11WIouYPBrIKY/edit?usp=sharing".to_string()),
            link_text: "Read Report".to_string(),
            icon: "<svg fill='none' stroke='currentColor' viewBox='0 0 24 24'><path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19.428 15.428a2 2 0 00-1.022-.547l-2.384-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z'></path></svg>".to_string(),
        },
        Artifact {
            title: "GitHub Repository".to_string(),
            description: "The complete Rust source code for the privacy-first creator's sandbox.".to_string(),
            tags: vec!["Rust".to_string(), "Axum".to_string(), "Leptos".to_string()],
            link: Some("https://github.com/joshua42atkinson/day_dream".to_string()),
            link_text: "View Code".to_string(),
            icon: "<svg fill='currentColor' viewBox='0 0 24 24'><path d='M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z'/></svg>".to_string(),
        },
    ];

    view! {
        <div class="max-w-6xl mx-auto space-y-12 animate-fade-in">
            // Hero Section
            <div class="text-center space-y-6">
                <div class="inline-flex items-center px-4 py-2 rounded-full bg-purple-900/30 border border-purple-500/50 text-purple-300 text-sm font-bold uppercase tracking-widest">
                    <span class="w-2 h-2 bg-purple-400 rounded-full mr-3 animate-pulse"></span>
                    "Capstone Project"
                </div>
                <h1 class="text-5xl md:text-7xl font-black text-white tracking-tight">
                    "The Daydream" <span class="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-600">"Initiative"</span>
                </h1>
                <p class="text-xl text-slate-300 max-w-3xl mx-auto leading-relaxed">
                    "A privacy-first 'creator's sandbox' empowering instructional designers to build narrative-driven intelligent tutoring systems."
                </p>
            </div>

            // Video Showcase
            <GlassPanel>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-8 items-center">
                    <div class="space-y-4">
                        <h2 class="text-2xl font-bold text-white">"The Engine of Enjoyment"</h2>
                        <p class="text-slate-400">
                            "How do we solve the 'Edutainment Gap'? By fusing the narrative structure of the Hero's Journey with the rigorous scaffolding of Cognitive Load Theory."
                        </p>
                        <div class="flex gap-4 pt-4">
                             <a href="https://youtu.be/dYxmWd50xgs" target="_blank" class="px-6 py-3 rounded-lg bg-white/10 hover:bg-white/20 text-white font-semibold transition-all">
                                "Watch Tech Demo"
                             </a>
                        </div>
                    </div>
                    <div class="aspect-video rounded-lg bg-black/50 border border-white/10 flex items-center justify-center group cursor-pointer">
                        // Placeholder for video embed
                        <div class="text-center">
                            <div class="w-16 h-16 rounded-full bg-white/10 flex items-center justify-center mx-auto mb-4 group-hover:bg-purple-600 transition-colors">
                                <svg class="w-8 h-8 text-white ml-1" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
                            </div>
                            <span class="text-sm text-slate-400 uppercase tracking-widest">"Play Video"</span>
                        </div>
                    </div>
                </div>
            </GlassPanel>

            // Artifact Grid
            <div class="space-y-6">
                <h2 class="text-2xl font-bold text-white pl-2 border-l-4 border-purple-500">"Project Artifacts"</h2>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {artifacts.into_iter().map(|artifact| view! {
                        <ArtifactCard artifact=artifact />
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}