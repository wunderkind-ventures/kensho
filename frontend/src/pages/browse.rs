use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::components::{SearchBar, AnimeGrid, NavBar};
use crate::services::api::ApiClient;
use crate::models::AnimeSummary;

#[component]
pub fn Browse(cx: Scope, year: i32, season: String) -> Element {
    let router = use_navigator(cx);
    let seasonal_anime = use_state(cx, || Vec::<AnimeSummary>::new());
    let is_loading = use_state(cx, || true);
    let error = use_state(cx, || None::<String>);
    
    // Load seasonal anime
    use_effect(cx, (&year, &season.clone()), |(y, s)| {
        to_owned![seasonal_anime, is_loading, error];
        async move {
            is_loading.set(true);
            error.set(None);
            
            let api = ApiClient::new();
            match api.browse_seasonal(y, &s).await {
                Ok(anime) => {
                    seasonal_anime.set(anime);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load seasonal anime: {}", e)));
                }
            }
            
            is_loading.set(false);
        }
    });
    
    // Season navigation
    let prev_season = get_prev_season(year, &season);
    let next_season = get_next_season(year, &season);
    
    render! {
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
                    
                    // Navigation back to home
                    div { 
                        style: "margin-bottom: 2rem;",
                        Link {
                            to: "/",
                            style: "
                                color: #64c8ff;
                                text-decoration: none;
                                display: inline-flex;
                                align-items: center;
                                font-size: 1rem;
                                transition: opacity 0.2s;
                            ",
                            "← Back to Home"
                        }
                    }
                    
                    // Title
                    div { 
                        style: "text-align: center; margin-bottom: 2rem;",
                        h1 { 
                            style: "font-size: 2.5rem; font-weight: 700; margin-bottom: 0.5rem;",
                            "{format_season(&season)} {year}"
                        }
                        p { 
                            style: "color: #999; font-size: 1.1rem;",
                            "Browse seasonal anime releases"
                        }
                    }
                    
                    // Season navigation
                    div {
                        style: "display: flex; justify-content: center; gap: 2rem; margin-bottom: 2rem;",
                        
                        Link {
                            to: &format!("/browse/{}/{}", prev_season.0, prev_season.1),
                            style: "
                                background: #1a1a2e;
                                border: 1px solid #333;
                                color: white;
                                padding: 0.75rem 1.5rem;
                                border-radius: 8px;
                                text-decoration: none;
                                display: flex;
                                align-items: center;
                                gap: 0.5rem;
                                transition: background 0.2s;
                            ",
                            "← {format_season(&prev_season.1)} {prev_season.0}"
                        }
                        
                        div {
                            style: "
                                padding: 0.75rem 1.5rem;
                                background: rgba(100, 200, 255, 0.1);
                                border: 1px solid #64c8ff;
                                color: #64c8ff;
                                border-radius: 8px;
                                font-weight: 600;
                            ",
                            "Current Season"
                        }
                        
                        Link {
                            to: &format!("/browse/{}/{}", next_season.0, next_season.1),
                            style: "
                                background: #1a1a2e;
                                border: 1px solid #333;
                                color: white;
                                padding: 0.75rem 1.5rem;
                                border-radius: 8px;
                                text-decoration: none;
                                display: flex;
                                align-items: center;
                                gap: 0.5rem;
                                transition: background 0.2s;
                            ",
                            "{format_season(&next_season.1)} {next_season.0} →"
                        }
                    }
                    
                    // Search bar
                    div { 
                        style: "display: flex; justify-content: center;",
                        SearchBar {
                            placeholder: "Search anime...",
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
                                "Loading seasonal anime..."
                            }
                        }
                    }
                } else if let Some(error_msg) = error.get() {
                    rsx! {
                        div { 
                            style: "text-align: center; padding: 4rem;",
                            div { style: "font-size: 3rem; margin-bottom: 1rem;", "⚠" }
                            p { style: "color: #ff6464; font-size: 1.1rem;", "{error_msg}" }
                        }
                    }
                } else if seasonal_anime.get().is_empty() {
                    rsx! {
                        div { 
                            style: "text-align: center; padding: 4rem; background: #1a1a2e; border-radius: 8px;",
                            div { style: "font-size: 3rem; margin-bottom: 1rem; opacity: 0.5;", "📭" }
                            p { style: "color: #666; font-size: 1.1rem;", "No anime found for this season" }
                        }
                    }
                } else {
                    rsx! {
                        div {
                            // Results count
                            div { 
                                style: "margin-bottom: 1.5rem;",
                                p { 
                                    style: "color: #999;",
                                    "Found {seasonal_anime.get().len()} anime for {format_season(&season)} {year}"
                                }
                            }
                            
                            // Anime grid
                            AnimeGrid { anime_list: seasonal_anime.get() }
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

fn format_season(season: &str) -> &str {
    match season.to_lowercase().as_str() {
        "winter" => "Winter",
        "spring" => "Spring",
        "summer" => "Summer",
        "fall" => "Fall",
        _ => season,
    }
}

fn get_prev_season(year: i32, season: &str) -> (i32, String) {
    match season.to_lowercase().as_str() {
        "spring" => (year, "winter".to_string()),
        "summer" => (year, "spring".to_string()),
        "fall" => (year, "summer".to_string()),
        "winter" => (year - 1, "fall".to_string()),
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