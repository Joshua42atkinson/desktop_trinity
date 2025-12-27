use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    *,
};

use crate::components::ui::draggable_window::DraggableWindow;
use crate::components::ui::zen_toggle::ZenToggle;
use crate::components::workflow::graph_editor::GraphEditor;
use crate::pages::autonomous::AutonomousDashboard;
use crate::pages::chat::ChatPage;
use crate::pages::dashboard::TrinityDashboard;
use crate::pages::memory::MemoryPage;
use crate::pages::not_found::NotFound;
use crate::pages::notebook::NotebookPage;
// Creative Studios
use crate::pages::game_studio::GameStudio;
use crate::pages::studios::StudiosHub;
use crate::pages::video_studio::VideoStudio;
use crate::pages::web_studio::WebStudio;
use crate::pages::writing_studio::WritingStudio;

#[component]
pub fn App() -> impl IntoView {
    // Initialize Global UI State
    provide_context(crate::ui_state::GlobalUiState::new());

    view! {
        <Layout/>
    }
}

/// Navigation link with active state styling
#[component]
fn NavLink(
    href: &'static str,
    label: &'static str,
    color: &'static str,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] pulse: bool,
) -> impl IntoView {
    let location = use_location();
    let is_active = move || {
        let path = location.pathname.get();
        if href == "/" {
            path == "/"
        } else {
            path.starts_with(href)
        }
    };

    let base_classes = "px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 flex items-center gap-1";

    view! {
        <a
            href=href
            class=move || {
                if is_active() {
                    format!("{} bg-white/10 {}", base_classes, color)
                } else {
                    format!("{} text-gray-300 hover:{} hover:bg-white/5", base_classes, color)
                }
            }
        >
            {icon.map(|i| view! { <span>{i}</span> })}
            {pulse.then(|| view! { <span class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span> })}
            {label}
        </a>
    }
}

#[component]
fn Layout() -> impl IntoView {
    let (_is_menu_open, _set_is_menu_open) = signal(false);
    let (mobile_menu_open, set_mobile_menu_open) = signal(false);
    let state = use_context::<crate::ui_state::GlobalUiState>().expect("GlobalUiState missing");
    let zen = state.zen_mode;

    view! {
        <div class="relative min-h-screen font-inter overflow-hidden selection:bg-cyan-500 selection:text-white">
            // Zen Toggle
            <ZenToggle/>

            // Artifact HUD
            <DraggableWindow
                title="Artifact HUD".to_string()
                initial_visible=false
            >
                {move || {
                    match state.hud_content.get() {
                        crate::ui_state::HudContent::Empty => view! {
                            <div class="p-8 text-center text-gray-500 italic flex flex-col items-center">
                                <div class="text-4xl mb-2 opacity-30">"📦"</div>
                                "Select an artifact to view"
                            </div>
                        }.into_any(),
                        crate::ui_state::HudContent::Video(url) => view! {
                            <div class="aspect-video w-full bg-black rounded overflow-hidden shadow-inner">
                                <iframe
                                    src=url
                                    class="w-full h-full"
                                    allow="autoplay; encrypted-media"
                                    allowfullscreen
                                ></iframe>
                            </div>
                        }.into_any(),
                        crate::ui_state::HudContent::Code(content, lang) => view! {
                            <div class="bg-gray-950 p-4 rounded-lg font-mono text-xs overflow-auto h-full border border-white/10 shadow-inner">
                                <div class="flex justify-between items-center mb-2 border-b border-white/10 pb-2">
                                    <span class="text-cyan-400 font-bold uppercase">{lang}</span>
                                    <button class="text-gray-500 hover:text-white transition-colors">"Copy"</button>
                                </div>
                                <pre class="text-gray-300 whitespace-pre-wrap">{content}</pre>
                            </div>
                        }.into_any(),
                    }
                }}
            </DraggableWindow>

            // Aurora Background
            <div class="fixed inset-0 z-0 bg-slate-900">
                <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-900 via-[#0a0a0a] to-black"></div>
                <div class="absolute inset-0 bg-[url('/noise.svg')] opacity-20 mix-blend-soft-light"></div>
            </div>

            // Navbar (Hidden in Zen Mode)
            <nav class="sticky top-0 z-50 w-full border-b border-white/10 bg-slate-900/70 backdrop-blur-md transition-all duration-500"
                 class:opacity-0={move || zen.get()}
                 class:pointer-events-none={move || zen.get()}
                 class:-translate-y-full={move || zen.get()}
            >
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="flex items-center justify-between h-16">
                        <div class="flex-shrink-0">
                            <a href="/" class="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 via-purple-500 to-amber-400">
                                "Trinity AI OS"
                            </a>
                        </div>

                        // Mobile menu button
                        <div class="md:hidden">
                            <button
                                class="text-gray-300 hover:text-white p-2"
                                on:click=move |_| set_mobile_menu_open.update(|v| *v = !*v)
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    {move || if mobile_menu_open.get() {
                                        view! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path> }.into_any()
                                    } else {
                                        view! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path> }.into_any()
                                    }}
                                </svg>
                            </button>
                        </div>

                        // Desktop Navigation
                        <div class="hidden md:block">
                            <div class="ml-10 flex items-baseline space-x-2">
                                <NavLink href="/" label="Dashboard" color="text-blue-400"/>
                                <NavLink href="/autonomous" label="Autonomous" color="text-purple-400" pulse=true/>
                                <NavLink href="/studios" label="Studios" color="text-rose-400"/>
                                <NavLink href="/notebook" label="Notebook" color="text-purple-400"/>
                                <NavLink href="/agents" label="Agents" color="text-amber-400"/>
                                <NavLink href="/memory" label="Memory" color="text-green-400"/>
                                <NavLink href="/chat" label="Chat" color="text-cyan-400"/>
                            </div>
                        </div>
                    </div>

                    // Mobile Navigation Menu
                    <div
                        class="md:hidden overflow-hidden transition-all duration-300"
                        class:max-h-0={move || !mobile_menu_open.get()}
                        class:max-h-96={move || mobile_menu_open.get()}
                    >
                        <div class="py-2 space-y-1">
                            <a href="/" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md">"Dashboard"</a>
                            <a href="/autonomous" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md flex items-center gap-2">
                                <span class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span>
                                "Autonomous"
                            </a>
                            <a href="/studios" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md">"Studios"</a>
                            <a href="/notebook" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md">"Notebook"</a>
                            <a href="/agents" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md">"Agents"</a>
                            <a href="/memory" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md">"Memory"</a>
                            <a href="/chat" class="block px-3 py-2 text-gray-300 hover:bg-white/5 rounded-md">"Chat"</a>
                        </div>
                    </div>
                </div>
            </nav>

            <main class="relative z-10">
                <Router>
                    <Routes fallback=|| view! { "Page Not Found" }>
                        <Route path=path!("/") view=TrinityDashboard/>
                        <Route path=path!("/autonomous") view=AutonomousDashboard/>
                        <Route path=path!("/studios") view=StudiosHub/>
                        <Route path=path!("/studios/video") view=VideoStudio/>
                        <Route path=path!("/studios/games") view=GameStudio/>
                        <Route path=path!("/studios/web") view=WebStudio/>
                        <Route path=path!("/studios/writing") view=WritingStudio/>
                        <Route path=path!("/notebook") view=NotebookPage/>
                        <Route path=path!("/agents") view=GraphEditor/>
                        <Route path=path!("/memory") view=MemoryPage/>
                        <Route path=path!("/chat") view=ChatPage/>
                        <Route path=path!("/*any") view=NotFound/>
                    </Routes>
                </Router>
            </main>
        </div>
    }
}
