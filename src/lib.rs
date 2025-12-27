use leptos::*;
use leptos_router::*;

mod models;
mod components;
mod pages;

use pages::home::Home;
use pages::foundations::Foundations;
use pages::planning::Planning;
use pages::design::Design;
use pages::evaluation::Evaluation;
use pages::daydream::Daydream;
use pages::about::About;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Layout>
                <Routes>
                    <Route path="/" view=Home/>
                    <Route path="/foundations" view=Foundations/>
                    <Route path="/planning" view=Planning/>
                    <Route path="/design" view=Design/>
                    <Route path="/evaluation" view=Evaluation/>
                    <Route path="/daydream" view=Daydream/>
                    <Route path="/about" view=About/>
                    <Route path="/*any" view=|| view! { <div class="text-center p-10">"Page Construction in Progress"</div> }/>
                </Routes>
            </Layout>
        </Router>
    }
}

/// The Main Layout component that wraps every page.
/// It contains the persistent Aurora Background and the Navigation Bar.
#[component]
fn Layout(children: Children) -> impl IntoView {
    // State for mobile menu toggle
    let (is_menu_open, set_is_menu_open) = create_signal(false);

    view! {
        <div class="relative min-h-screen font-inter overflow-hidden selection:bg-cyan-500 selection:text-white">

            // --- 1. The "Daydream" Aurora Background Animation ---
            <div class="fixed inset-0 z-0 pointer-events-none">
                <div class="absolute top-0 left-1/4 w-96 h-96 bg-cyan-500/20 rounded-full mix-blend-screen filter blur-3xl opacity-30 animate-blob"></div>
                <div class="absolute top-0 right-1/4 w-96 h-96 bg-blue-600/20 rounded-full mix-blend-screen filter blur-3xl opacity-30 animate-blob animation-delay-2000"></div>
                <div class="absolute -bottom-32 left-1/3 w-96 h-96 bg-purple-600/20 rounded-full mix-blend-screen filter blur-3xl opacity-30 animate-blob animation-delay-4000"></div>
            </div>

            // --- 2. Glassmorphism Navbar ---
            <nav class="sticky top-0 z-50 w-full border-b border-white/10 bg-slate-900/70 backdrop-blur-md">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="flex items-center justify-between h-16">
                        <div class="flex-shrink-0">
                            <span class="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 to-blue-500">
                                "LDT Portfolio"
                            </span>
                        </div>

                        // Desktop Menu
                        <div class="hidden md:block">
                            <div class="ml-10 flex items-baseline space-x-4">
                                <NavLink href="/" text="Home"/>
                                <NavLink href="/foundations" text="Foundations"/>
                                <NavLink href="/planning" text="Planning"/>
                                <NavLink href="/design" text="Design"/>
                                <NavLink href="/evaluation" text="Evaluation"/>
                                <NavLink href="/daydream" text="Daydream"/>
                                <NavLink href="/about" text="About"/>
                            </div>
                        </div>

                        // Mobile Menu Button
                        <div class="-mr-2 flex md:hidden">
                            <button
                                on:click=move |_| set_is_menu_open.update(|v| *v = !*v)
                                type="button"
                                class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none"
                            >
                                <svg class="h-6 w-6" stroke="currentColor" fill="none" viewBox="0 0 24 24">
                                    <path class:hidden=move || is_menu_open.get() stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                    <path class:hidden=move || !is_menu_open.get() stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>

                // Mobile Menu Dropdown
                <div class:hidden=move || !is_menu_open.get() class="md:hidden bg-slate-900 border-b border-white/10">
                    <div class="px-2 pt-2 pb-3 space-y-1 sm:px-3">
                         <MobileNavLink href="/" text="Home"/>
                         <MobileNavLink href="/foundations" text="Foundations"/>
                         <MobileNavLink href="/planning" text="Planning"/>
                         <MobileNavLink href="/design" text="Design"/>
                         <MobileNavLink href="/evaluation" text="Evaluation"/>
                         <MobileNavLink href="/daydream" text="Daydream"/>
                         <MobileNavLink href="/about" text="About"/>
                    </div>
                </div>
            </nav>

            // --- 3. Main Content Area ---
            <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                {children()}
            </div>
        </div>
    }
}

// Helper component for Desktop Navigation Links
#[component]
fn NavLink(href: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <A href=href class="text-gray-300 hover:text-cyan-400 hover:bg-white/5 px-3 py-2 rounded-md text-sm font-medium transition-all duration-200">
            {text}
        </A>
    }
}

// Helper component for Mobile Navigation Links
#[component]
fn MobileNavLink(href: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <A href=href class="text-gray-300 hover:text-white hover:bg-gray-700 block px-3 py-2 rounded-md text-base font-medium">
            {text}
        </A>
    }
}