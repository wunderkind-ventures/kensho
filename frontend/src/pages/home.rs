use dioxus::prelude::*;
use crate::components::{SearchBar, AnimeGrid, NavBar};
use crate::services::api::ApiClient;
use crate::models::AnimeSummary;

#[component]
pub fn Home() -> Element {
    let mut recent_anime = use_signal(|| Vec::<AnimeSummary>::new());
    let mut popular_anime = use_signal(|| Vec::<AnimeSummary>::new());
    let mut is_loading = use_signal(|| true);
    
    // Load initial data
    use_effect(move || {
        spawn(async move {
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
        });
    });
    
    rsx! {
        div { class: "home-page",
            style: "min-height: 100vh; background: #0a0a0a;",
            
            // Navigation bar
            NavBar {}
            
            // Header
            header {
                style: "
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    padding: 4rem 2rem;
                    text-align: center;
                ",
                div {
                    style: "max-width: 1200px; margin: 0 auto;",
                    h1 {
                        style: "
                            font-size: 3rem;
                            font-weight: bold;
                            color: white;
                            margin-bottom: 1rem;
                        ",
                        "見 Kenshō"
                    }
                    p {
                        style: "
                            font-size: 1.25rem;
                            color: rgba(255, 255, 255, 0.9);
                            margin-bottom: 2rem;
                        ",
                        "Discover and stream your favorite anime"
                    }
                    
                    // Search bar
                    div {
                        style: "max-width: 600px; margin: 0 auto;",
                        SearchBar {}
                    }
                }
            }
            
            // Main content
            main {
                style: "padding: 4rem 2rem; max-width: 1400px; margin: 0 auto;",
                
                if *is_loading.read() {
                    div {
                        style: "text-align: center; padding: 4rem;",
                        div {
                            style: "
                                display: inline-block;
                                width: 50px;
                                height: 50px;
                                border: 3px solid rgba(255,255,255,0.3);
                                border-radius: 50%;
                                border-top-color: #667eea;
                                animation: spin 1s ease-in-out infinite;
                            ",
                        }
                    }
                } else {
                    // Recent releases section
                    section {
                        style: "margin-bottom: 4rem;",
                        h2 {
                            style: "
                                font-size: 2rem;
                                font-weight: 600;
                                color: white;
                                margin-bottom: 2rem;
                            ",
                            "Recent Releases"
                        }
                        AnimeGrid { anime: recent_anime.read().clone() }
                    }
                    
                    // Popular anime section
                    section {
                        h2 {
                            style: "
                                font-size: 2rem;
                                font-weight: 600;
                                color: white;
                                margin-bottom: 2rem;
                            ",
                            "Popular This Season"
                        }
                        AnimeGrid { anime: popular_anime.read().clone() }
                    }
                }
            }
        }
    }
}