use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::components::{NavBar, VideoPlayer, EpisodeList};
use crate::services::api::ApiClient;
use crate::models::{Anime, Episode};

#[component]
pub fn Series(id: String) -> Element {
    let mut anime = use_signal(|| None::<Anime>);
    let mut episodes = use_signal(|| Vec::<Episode>::new());
    let mut selected_episode = use_signal(|| None::<Episode>);
    let mut is_loading = use_signal(|| true);
    let mut current_stream = use_signal(|| None::<String>);
    
    // Load anime data
    use_effect(move || {
        let anime_id = id.clone();
        spawn(async move {
            let api = ApiClient::new();
            
            // Load anime details
            match api.get_anime(&anime_id).await {
                Ok(anime_data) => {
                    anime.set(Some(anime_data));
                }
                Err(e) => {
                    tracing::error!("Failed to load anime: {}", e);
                }
            }
            
            // Load episodes
            match api.get_episodes(&anime_id).await {
                Ok(eps) => {
                    if !eps.is_empty() {
                        selected_episode.set(Some(eps[0].clone()));
                    }
                    episodes.set(eps);
                }
                Err(e) => {
                    tracing::error!("Failed to load episodes: {}", e);
                }
            }
            
            is_loading.set(false);
        });
    });
    
    rsx! {
        div { class: "series-page",
            style: "min-height: 100vh; background: #0a0a0a;",
            
            // Navigation bar
            NavBar {}
            
            // Main content
            if *is_loading.read() {
                div {
                    style: "display: flex; justify-content: center; align-items: center; height: 80vh;",
                    div {
                        style: "
                            width: 50px;
                            height: 50px;
                            border: 3px solid rgba(255,255,255,0.3);
                            border-radius: 50%;
                            border-top-color: #667eea;
                            animation: spin 1s ease-in-out infinite;
                        ",
                    }
                }
            } else if let Some(anime_data) = anime.read().as_ref() {
                div {
                    style: "max-width: 1400px; margin: 0 auto; padding: 2rem;",
                    
                    // Hero section with anime info
                    div {
                        style: "
                            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
                            border-radius: 12px;
                            padding: 2rem;
                            margin-bottom: 2rem;
                            display: grid;
                            grid-template-columns: 300px 1fr;
                            gap: 2rem;
                        ",
                        
                        // Poster
                        img {
                            src: {anime_data.poster_url.clone()},
                            alt: {anime_data.title.clone()},
                            style: "
                                width: 100%;
                                border-radius: 8px;
                                box-shadow: 0 10px 30px rgba(0,0,0,0.5);
                            ",
                        }
                        
                        // Info
                        div {
                            h1 {
                                style: "
                                    font-size: 2.5rem;
                                    font-weight: 700;
                                    color: white;
                                    margin-bottom: 1rem;
                                ",
                                {anime_data.title.clone()}
                            }
                            
                            p {
                                style: "
                                    color: #a0a0b0;
                                    line-height: 1.6;
                                    margin-bottom: 1.5rem;
                                ",
                                {anime_data.description.clone()}
                            }
                            
                            div {
                                style: "display: flex; gap: 1rem; flex-wrap: wrap;",
                                
                                span {
                                    style: "
                                        background: rgba(102, 126, 234, 0.1);
                                        border: 1px solid rgba(102, 126, 234, 0.3);
                                        color: #667eea;
                                        padding: 0.5rem 1rem;
                                        border-radius: 20px;
                                        font-size: 0.875rem;
                                    ",
                                    {format!("{} Episodes", anime_data.episode_count)}
                                }
                                
                                span {
                                    style: "
                                        background: rgba(168, 85, 247, 0.1);
                                        border: 1px solid rgba(168, 85, 247, 0.3);
                                        color: #a855f7;
                                        padding: 0.5rem 1rem;
                                        border-radius: 20px;
                                        font-size: 0.875rem;
                                    ",
                                    {anime_data.status.clone()}
                                }
                                
                                if let Some(rating) = anime_data.rating {
                                    span {
                                        style: "
                                            background: rgba(34, 197, 94, 0.1);
                                            border: 1px solid rgba(34, 197, 94, 0.3);
                                            color: #22c55e;
                                            padding: 0.5rem 1rem;
                                            border-radius: 20px;
                                            font-size: 0.875rem;
                                        ",
                                        {format!("⭐ {:.1}", rating)}
                                    }
                                }
                            }
                        }
                    }
                    
                    // Video player section
                    if let Some(stream_url) = current_stream.read().as_ref() {
                        div {
                            style: "margin-bottom: 2rem;",
                            VideoPlayer { stream_url: stream_url.clone() }
                        }
                    }
                    
                    // Episodes section
                    div {
                        style: "
                            background: rgba(26, 26, 46, 0.5);
                            border-radius: 12px;
                            padding: 1.5rem;
                        ",
                        
                        h2 {
                            style: "
                                font-size: 1.5rem;
                                font-weight: 600;
                                color: white;
                                margin-bottom: 1rem;
                            ",
                            "Episodes"
                        }
                        
                        div {
                            style: "
                                display: grid;
                                grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
                                gap: 1rem;
                            ",
                            
                            // Use the EpisodeList component
                            EpisodeList {
                                episodes: episodes.read().clone(),
                                on_select: move |ep: Episode| {
                                    selected_episode.set(Some(ep.clone()));
                                    current_stream.set(Some(format!("https://example.com/stream/{}", ep.id)));
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    style: "
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        height: 80vh;
                        color: #a0a0b0;
                    ",
                    p { "Anime not found" }
                    Link {
                        to: "/",
                        style: "
                            color: #667eea;
                            text-decoration: none;
                            margin-top: 1rem;
                        ",
                        "Return to Home"
                    }
                }
            }
        }
    }
}