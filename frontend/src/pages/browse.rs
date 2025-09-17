use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::components::{SearchBar, AnimeGrid, NavBar};
use crate::services::api::ApiClient;
use crate::models::AnimeSummary;

#[component]
pub fn Browse(year: i32, season: String) -> Element {
    let mut anime_list = use_signal(|| Vec::<AnimeSummary>::new());
    let mut is_loading = use_signal(|| true);
    let nav = navigator();
    
    // Load seasonal anime
    use_effect(move || {
        let year = year;
        let season = season.clone();
        spawn(async move {
            let api = ApiClient::new();
            
            match api.browse_seasonal(year, &season).await {
                Ok(anime) => {
                    anime_list.set(anime);
                }
                Err(e) => {
                    tracing::error!("Failed to load seasonal anime: {}", e);
                }
            }
            
            is_loading.set(false);
        });
    });
    
    // Season navigation helpers
    let (prev_year, prev_season) = get_prev_season(year, &season);
    let (next_year, next_season) = get_next_season(year, &season);
    
    rsx! {
        div { class: "browse-page",
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
                    
                    h1 {
                        style: "
                            font-size: 2rem;
                            font-weight: 600;
                            color: white;
                            margin-bottom: 1rem;
                        ",
                        {format!("{} {} Anime", season_display_name(&season), year)}
                    }
                    
                    // Season navigation
                    div {
                        style: "display: flex; gap: 1rem; align-items: center;",
                        
                        Link {
                            to: format!("/browse/{}/{}", prev_year, prev_season),
                            style: "
                                padding: 0.5rem 1rem;
                                background: rgba(255,255,255,0.1);
                                color: white;
                                border-radius: 8px;
                                text-decoration: none;
                            ",
                            "← Previous"
                        }
                        
                        span {
                            style: "color: #a0a0b0;",
                            "Navigate Seasons"
                        }
                        
                        Link {
                            to: format!("/browse/{}/{}", next_year, next_season),
                            style: "
                                padding: 0.5rem 1rem;
                                background: rgba(255,255,255,0.1);
                                color: white;
                                border-radius: 8px;
                                text-decoration: none;
                            ",
                            "Next →"
                        }
                    }
                }
            }
            
            // Search bar
            div {
                style: "max-width: 600px; margin: 2rem auto;",
                SearchBar {}
            }
            
            // Main content
            main {
                style: "padding: 2rem; max-width: 1400px; margin: 0 auto;",
                
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
                } else if anime_list.read().is_empty() {
                    div {
                        style: "
                            text-align: center;
                            padding: 4rem;
                            color: #a0a0b0;
                        ",
                        p { "No anime found for this season." }
                        Link {
                            to: "/",
                            style: "
                                color: #667eea;
                                text-decoration: none;
                            ",
                            "Return to Home"
                        }
                    }
                } else {
                    AnimeGrid { anime: anime_list.read().clone() }
                }
            }
        }
    }
}

fn get_prev_season(year: i32, season: &str) -> (i32, String) {
    match season.to_lowercase().as_str() {
        "winter" => (year - 1, "fall".to_string()),
        "spring" => (year, "winter".to_string()),
        "summer" => (year, "spring".to_string()),
        "fall" => (year, "summer".to_string()),
        _ => (year, "winter".to_string()),
    }
}

fn get_next_season(year: i32, season: &str) -> (i32, String) {
    match season.to_lowercase().as_str() {
        "winter" => (year, "spring".to_string()),
        "spring" => (year, "summer".to_string()),
        "summer" => (year, "fall".to_string()),
        "fall" => (year + 1, "winter".to_string()),
        _ => (year, "spring".to_string()),
    }
}

fn season_display_name(season: &str) -> &str {
    match season.to_lowercase().as_str() {
        "winter" => "Winter",
        "spring" => "Spring",
        "summer" => "Summer",
        "fall" => "Fall",
        _ => season,
    }
}