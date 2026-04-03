mod api;
mod models;
mod theme;
mod components;
mod pages;
mod state;

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use pages::*;
use state::AppState;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Warn).ok();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let app_state = AppState::new();
    provide_context(app_state);

    view! {
        <Router>
            <Layout>
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=dashboard::DashboardPage />
                    <Route path=path!("/heart") view=heart::HeartPage />
                    <Route path=path!("/sleep") view=sleep::SleepPage />
                    <Route path=path!("/activity") view=activity::ActivityPage />
                    <Route path=path!("/trends") view=trends::TrendsPage />
                    <Route path=path!("/settings") view=settings::SettingsPage />
                </Routes>
            </Layout>
        </Router>
    }
}

#[component]
fn Layout(children: Children) -> impl IntoView {
    let state = expect_context::<AppState>();
    let (mobile_nav_open, set_mobile_nav_open) = signal(false);

    // Bootstrap: load config and data
    Effect::new(move || {
        let s = state.clone();
        leptos::task::spawn_local(async move { s.bootstrap().await; });
    });

    view! {
        <div class="flex min-h-screen">
            // Desktop sidebar
            // Loading bar
            <Show when=move || state.loading.get()>
                <div class="fixed top-0 left-0 right-0 z-50 loading-bar"></div>
            </Show>

            <nav class="hidden md:flex flex-col w-[60px] hover:w-[200px] transition-all duration-200 border-r border-border bg-bg fixed h-full z-30 group overflow-hidden">
                <div class="p-3 mb-4 mt-4">
                    <div class="w-9 h-9 rounded-lg flex items-center justify-center glow-cyan">
                        <svg viewBox="0 0 24 24" class="w-5 h-5 text-accent" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="4,16 8,8 12,14 16,4 20,10" />
                        </svg>
                    </div>
                </div>
                <NavItems />
            </nav>

            // Main content
            <div class="flex-1 md:ml-[60px] pb-20 md:pb-0">
                // Header
                <header class="sticky top-0 z-20 border-b border-border bg-bg px-4 py-3">
                    <div class="flex items-center justify-between max-w-screen-2xl mx-auto">
                        <div class="flex items-center gap-3">
                            <button class="md:hidden p-2 text-dim hover:text-text"
                                on:click=move |_| set_mobile_nav_open.set(!mobile_nav_open.get())>
                                <svg viewBox="0 0 24 24" class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2">
                                    <line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" />
                                </svg>
                            </button>
                            <span class="font-bold text-accent text-sm uppercase tracking-[0.2em]" style="text-shadow: 0 0 12px rgba(0,240,255,0.5)">"GARMIN"</span>
                            // User switcher - always visible
                            {move || {
                                let users = state.users.get();
                                let current_uid = state.user_id.get();
                                if users.is_empty() {
                                    view! { <span></span> }.into_any()
                                } else if users.len() == 1 {
                                    let name = users.first().map(|u| {
                                        let n = &u.garmin_username;
                                        if n.contains('@') { n.split('@').next().unwrap_or(n).to_string() } else { n.clone() }
                                    }).unwrap_or_default();
                                    view! {
                                        <span class="text-accent text-sm font-display font-medium px-2 py-1 rounded-lg bg-surface border border-border">{name}</span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <select class="bg-surface border border-border rounded-lg px-2 py-1.5 text-sm text-accent font-display min-h-[36px]"
                                            on:change=move |e| {
                                                use wasm_bindgen::JsCast;
                                                let uid = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                                                let s = state;
                                                leptos::task::spawn_local(async move { s.switch_user(uid).await; });
                                            }>
                                            {users.iter().map(|u| {
                                                let selected = u.user_id == current_uid;
                                                let uid = u.user_id.clone();
                                                let name = if u.garmin_username.contains('@') {
                                                    u.garmin_username.split('@').next().unwrap_or(&u.garmin_username).to_string()
                                                } else { u.garmin_username.clone() };
                                                view! { <option value=uid selected=selected>{name}</option> }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    }.into_any()
                                }
                            }}
                            <span class="text-dim text-sm hidden sm:inline">{move || chrono::Utc::now().format("%a, %b %d").to_string()}</span>
                        </div>
                        <div class="flex items-center gap-2">
                            // Quick stats pills
                            {move || state.vitals.get().map(|v| view! {
                                <div class="hidden sm:flex items-center gap-2">
                                    {v.steps.map(|s| view! {
                                        <span class="pill border-steps/30 text-steps bg-steps/10">{format!("{} steps", s)}</span>
                                    })}
                                    {v.body_battery_high.map(|b| view! {
                                        <span class="pill border-good/30 text-good bg-good/10">{format!("{}% battery", b)}</span>
                                    })}
                                    {v.resting_heart_rate.map(|hr| view! {
                                        <span class="pill border-heart/30 text-heart bg-heart/10">{format!("{} bpm", hr)}</span>
                                    })}
                                </div>
                            })}
                            <a href="/settings" class="p-2 text-dim hover:text-text transition-colors">
                                <svg viewBox="0 0 24 24" class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2">
                                    <circle cx="12" cy="12" r="3" /><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
                                </svg>
                            </a>
                        </div>
                    </div>
                </header>

                // Mobile slide-out nav
                <Show when=move || mobile_nav_open.get()>
                    <div class="fixed inset-0 z-40 md:hidden">
                        <div class="absolute inset-0 bg-black/50" on:click=move |_| set_mobile_nav_open.set(false)></div>
                        <nav class="absolute left-0 top-0 h-full w-64 bg-bg border-r border-white/[0.06] p-4 animate-fade-in">
                            <div class="font-display font-bold text-accent text-lg mb-6">"Garmin Dashboard"</div>
                            <div on:click=move |_| set_mobile_nav_open.set(false)>
                                <NavItems />
                            </div>
                        </nav>
                    </div>
                </Show>

                // Page content
                <main class="p-4 sm:p-6 max-w-screen-2xl mx-auto animate-fade-in">
                    {children()}
                </main>
            </div>

            // Mobile bottom nav
            <nav class="md:hidden fixed bottom-0 left-0 right-0 z-30 border-t border-border bg-bg">
                <div class="flex justify-around py-2">
                    <BottomNavItem href="/" icon="home" label="Home" />
                    <BottomNavItem href="/heart" icon="heart" label="Heart" />
                    <BottomNavItem href="/sleep" icon="moon" label="Sleep" />
                    <BottomNavItem href="/activity" icon="activity" label="Activity" />
                    <BottomNavItem href="/trends" icon="trending" label="Trends" />
                </div>
            </nav>
        </div>
    }
}

#[component]
fn NavItems() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1 px-2">
            <a href="/" class="nav-item"><NavIcon icon="home" /><span class="md:opacity-0 md:group-hover:opacity-100 transition-opacity">"Dashboard"</span></a>
            <a href="/heart" class="nav-item"><NavIcon icon="heart" /><span class="md:opacity-0 md:group-hover:opacity-100 transition-opacity">"Heart & Body"</span></a>
            <a href="/sleep" class="nav-item"><NavIcon icon="moon" /><span class="md:opacity-0 md:group-hover:opacity-100 transition-opacity">"Sleep"</span></a>
            <a href="/activity" class="nav-item"><NavIcon icon="activity" /><span class="md:opacity-0 md:group-hover:opacity-100 transition-opacity">"Activity"</span></a>
            <a href="/trends" class="nav-item"><NavIcon icon="trending" /><span class="md:opacity-0 md:group-hover:opacity-100 transition-opacity">"Trends"</span></a>
            <a href="/settings" class="nav-item mt-auto"><NavIcon icon="settings" /><span class="md:opacity-0 md:group-hover:opacity-100 transition-opacity">"Settings"</span></a>
        </div>
    }
}

#[component]
fn NavIcon(icon: &'static str) -> impl IntoView {
    let path = match icon {
        "home" => "M3 12l9-9 9 9M5 10v10a1 1 0 001 1h3M19 10v10a1 1 0 01-1 1h-3M9 21v-6a1 1 0 011-1h4a1 1 0 011 1v6",
        "heart" => "M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z",
        "moon" => "M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z",
        "activity" => "M22 12h-4l-3 9L9 3l-3 9H2",
        "trending" => "M23 6l-9.5 9.5-5-5L1 18",
        "settings" => "M12 15a3 3 0 100-6 3 3 0 000 6zM19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z",
        _ => "",
    };
    view! {
        <svg viewBox="0 0 24 24" class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d=path />
        </svg>
    }
}

#[component]
fn BottomNavItem(href: &'static str, icon: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a href=href class="flex flex-col items-center gap-0.5 px-3 py-1 text-dim hover:text-accent transition-colors">
            <NavIcon icon=icon />
            <span class="text-[0.6rem]">{label}</span>
        </a>
    }
}

fn load_setting(key: &str, default: &str) -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
        .unwrap_or_else(|| default.to_string())
}

fn save_setting(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(key, value);
    }
}
