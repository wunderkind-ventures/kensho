use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::components::{IpHub, EpisodeList, VideoPlayer, NavBar};
use crate::services::api::ApiClient;
use crate::services::auth::use_auth;
use crate::models::{Anime, Episode};

#[component]
pub fn Series(cx: Scope, id: String) -> Element {
    let anime = use_state(cx, || None::<Anime>);
    let episodes = use_state(cx, || Vec::<Episode>::new());
    let current_stream = use_state(cx, || None::<String>);
    let is_loading = use_state(cx, || true);
    let error = use_state(cx, || None::<String>);
    let auth = use_auth(cx);
    let api = ApiClient::new();
    
    // Load anime and episodes data
    use_effect(cx, &id.clone(), |anime_id| {
        to_owned![anime, episodes, is_loading, error, api];
        async move {
            is_loading.set(true);
            error.set(None);
            
            // Fetch anime details
            match api.get_anime(&anime_id).await {
                Ok(anime_data) => {
                    anime.set(Some(anime_data));
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load anime: {}", e)));
                    is_loading.set(false);
                    return;
                }
            }
            
            // Fetch episodes
            match api.get_episodes(&anime_id).await {
                Ok(episodes_data) => {
                    episodes.set(episodes_data);
                }
                Err(e) => {
                    tracing::error!("Failed to load episodes: {}", e);
                    // Don't set error for episodes, still show anime info
                }
            }
            
            is_loading.set(false);
        }
    });
    
    // Handle episode play
    let handle_play = move |(anime_id, episode_num): (String, i32)| {
        if let Some(token) = auth.read().get_token() {
            let api_clone = api.clone();
            let current_stream_clone = current_stream.clone();
            let token = token.to_string();
            
            cx.spawn(async move {
                match api_clone.get_stream_url(&anime_id, episode_num, &token).await {
                    Ok(stream_data) => {
                        current_stream_clone.set(Some(stream_data.url));
                    }
                    Err(e) => {
                        tracing::error!("Failed to get stream URL: {}", e);
                        // Could show error toast here
                    }
                }
            });
        }
    };
    
    render! {
        div { class: "series-page",
            style: "min-height: 100vh; background: #0a0a0a;",
            
            // Navigation bar
            NavBar {}
            
            // Main content
            if *is_loading.get() {
                rsx! {
                    div { 
                        style: "display: flex; align-items: center; justify-content: center; min-height: 80vh;",
                        div {
                            style: "text-align: center;",
                            div {
                                style: "
                                    width: 50px;
                                    height: 50px;
                                    border: 3px solid #333;
                                    border-top-color: #64c8ff;
                                    border-radius: 50%;
                                    animation: spin 1s linear infinite;
                                    margin: 0 auto 1rem;
                                ",
                            }
                            p { style: "color: #666;", "Loading anime..." }
                        }
                    }
                }
            } else if let Some(error_msg) = error.get() {
                rsx! {
                    div { 
                        style: "display: flex; align-items: center; justify-content: center; min-height: 80vh;",
                        div {
                            style: "text-align: center; max-width: 400px;",
                            div { style: "font-size: 3rem; margin-bottom: 1rem;", "⚠" }
                            h2 { style: "font-size: 1.5rem; margin-bottom: 1rem;", "Error Loading Anime" }
                            p { style: "color: #999; margin-bottom: 2rem;", "{error_msg}" }
                            Link {
                                to: "/",
                                style: "
                                    display: inline-block;
                                    background: #1a1a2e;
                                    color: #64c8ff;
                                    padding: 0.75rem 1.5rem;
                                    border-radius: 8px;
                                    text-decoration: none;
                                ",
                                "← Back to Home"
                            }
                        }
                    }
                }
            } else if let Some(anime_data) = anime.get() {
                rsx! {
                    div {
                        // Video player (if streaming)
                        if let Some(stream_url) = current_stream.get() {
                            rsx! {
                                div {
                                    style: "max-width: 1400px; margin: 2rem auto; padding: 0 1rem;",
                                    div {
                                        style: "max-width: 900px; margin: 0 auto;",
                                        VideoPlayer {
                                            stream_url: stream_url,
                                            poster_url: anime_data.poster_url.as_deref(),
                                        }
                                    }
                                }
                            }
                        }
                        
                        // IP Hub
                        IpHub {
                            anime: anime_data,
                            episodes: episodes.get(),
                        }
                        
                        // Episodes section
                        div {
                            style: "max-width: 1400px; margin: 2rem auto; padding: 0 1rem;",
                            div {
                                style: "display: grid; grid-template-columns: 1fr; gap: 2rem;",
                                
                                // Episode list
                                EpisodeList {
                                    episodes: episodes.get(),
                                    anime_id: &id,
                                    onplay: move |data| handle_play(data),
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