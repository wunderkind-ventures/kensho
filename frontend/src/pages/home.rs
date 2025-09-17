use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::components::{SearchBar, AnimeGrid, NavBar};
use crate::services::api::ApiClient;
use crate::models::AnimeSummary;

#[component]
pub fn Home(cx: Scope) -> Element {
    let router = use_navigator(cx);
    let recent_anime = use_state(cx, || Vec::<AnimeSummary>::new());
    let popular_anime = use_state(cx, || Vec::<AnimeSummary>::new());
    let is_loading = use_state(cx, || true);
    
    // Load initial data
    use_effect(cx, (), |_| {
        to_owned![recent_anime, popular_anime, is_loading];
        async move {
            let api = ApiClient::new();
            
            // Load recent anime (current season)
            let current_year = 2025; // In production, get from Date
            let current_season = "spring"; // In production, calculate from month
            
            match api.browse_seasonal(current_year, current_season).await {
                Ok(anime) => {
                    recent_anime.set(anime);
                }
                Err(e) => {
                    tracing::error!("Failed to load recent anime: {}", e);
                }
            }
            
            // For now, use the same data for popular (in production, would have separate endpoint)
            match api.browse_seasonal(current_year, "winter").await {
                Ok(anime) => {
                    popular_anime.set(anime);
                }
                Err(e) => {
                    tracing::error!("Failed to load popular anime: {}", e);
                }
            }
            
            is_loading.set(false);
        }
    });
    
    render! {
        div { class: "home-page",
            style: "min-height: 100vh; background: #0a0a0a;",
            
            // Navigation bar
            NavBar {}
            
            // Header
            header {
                style: "
                    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
                    padding: 2rem 1rem;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.3);
                ",
                div { 
                    style: "max-width: 1200px; margin: 0 auto;",
                    
                    // Logo and title
                    div { 
                        style: "text-align: center; margin-bottom: 2rem;",
                        h1 { 
                            style: "
                                font-size: 3rem;
                                font-weight: 700;
                                background: linear-gradient(135deg, #64c8ff 0%, #a855f7 100%);
                                -webkit-background-clip: text;
                                -webkit-text-fill-color: transparent;
                                margin-bottom: 0.5rem;
                            ",
                            "Project Kenshō"
                        }
                        p { 
                            style: "color: #999; font-size: 1.1rem;",
                            "Discover your next favorite anime"
                        }
                    }
                    
                    // Search bar
                    div { 
                        style: "display: flex; justify-content: center;",
                        SearchBar {
                            placeholder: "Search for anime...",
                            onselect: move |anime| {
                                router.push(&format!("/anime/{}", anime.id));
                            }
                        }
                    }
                }
            }
            
            // Main content
            main {
                style: "max-width: 1200px; margin: 0 auto; padding: 2rem 1rem;",
                
                // Loading state
                if *is_loading.get() {
                    rsx! {
                        div { 
                            style: "text-align: center; padding: 4rem;",
                            div {
                                style: "
                                    width: 50px;
                                    height: 50px;
                                    border: 3px solid #333;
                                    border-top-color: #64c8ff;
                                    border-radius: 50%;
                                    animation: spin 1s linear infinite;
                                    margin: 0 auto;
                                ",
                            }
                            p { 
                                style: "margin-top: 1rem; color: #666;",
                                "Loading anime..."
                            }
                        }
                    }
                } else {
                    rsx! {
                        div {
                            // Recent releases section
                            section { 
                                style: "margin-bottom: 3rem;",
                                div { 
                                    style: "display: flex; justify-content: between; align-items: center; margin-bottom: 1.5rem;",
                                    h2 { 
                                        style: "font-size: 1.75rem; font-weight: 600;",
                                        "Recent Releases"
                                    }
                                    Link {
                                        to: "/browse/2025/spring",
                                        style: "
                                            color: #64c8ff;
                                            text-decoration: none;
                                            font-size: 1rem;
                                            transition: opacity 0.2s;
                                            margin-left: auto;
                                        ",
                                        "View all →"
                                    }
                                }
                                
                                if recent_anime.get().is_empty() {
                                    rsx! {
                                        div { 
                                            style: "text-align: center; padding: 3rem; background: #1a1a2e; border-radius: 8px;",
                                            p { style: "color: #666;", "No recent anime available" }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        AnimeGrid { anime_list: recent_anime.get() }
                                    }
                                }
                            }
                            
                            // Popular section
                            section {
                                div { 
                                    style: "display: flex; justify-content: between; align-items: center; margin-bottom: 1.5rem;",
                                    h2 { 
                                        style: "font-size: 1.75rem; font-weight: 600;",
                                        "Popular This Season"
                                    }
                                    Link {
                                        to: "/browse/2025/winter",
                                        style: "
                                            color: #64c8ff;
                                            text-decoration: none;
                                            font-size: 1rem;
                                            transition: opacity 0.2s;
                                            margin-left: auto;
                                        ",
                                        "View all →"
                                    }
                                }
                                
                                if popular_anime.get().is_empty() {
                                    rsx! {
                                        div { 
                                            style: "text-align: center; padding: 3rem; background: #1a1a2e; border-radius: 8px;",
                                            p { style: "color: #666;", "No popular anime available" }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        AnimeGrid { anime_list: popular_anime.get() }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        style { 
            "@keyframes spin {{
                to {{ transform: rotate(360deg); }}
            }}"
        }
    }
}