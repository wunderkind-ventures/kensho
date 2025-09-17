use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::models::AnimeSummary;

#[component]
pub fn AnimeCard(anime: AnimeSummary) -> Element {
    let nav = navigator();
    let anime_id = anime.id.clone();
    
    rsx! {
        div { 
            class: "anime-card",
            onclick: move |_| nav.push(format!("/anime/{}", anime_id)),
            style: "
                background: rgba(26, 26, 46, 0.6);
                border-radius: 12px;
                overflow: hidden;
                cursor: pointer;
                transition: all 0.3s;
            ",
            
            // Poster image
            div {
                style: "
                    position: relative;
                    aspect-ratio: 3/4;
                    overflow: hidden;
                ",
                img {
                    src: {anime.poster_url.clone()},
                    alt: {anime.title.clone()},
                    style: "
                        width: 100%;
                        height: 100%;
                        object-fit: cover;
                    ",
                }
                
                // Status badge
                div {
                    style: "
                        position: absolute;
                        top: 0.5rem;
                        right: 0.5rem;
                        background: rgba(102, 126, 234, 0.9);
                        color: white;
                        padding: 0.25rem 0.75rem;
                        border-radius: 20px;
                        font-size: 0.75rem;
                        font-weight: 600;
                    ",
                    {anime.status.clone()}
                }
            }
            
            // Card info
            div {
                style: "padding: 1rem;",
                
                h3 {
                    style: "
                        color: white;
                        font-size: 1rem;
                        font-weight: 600;
                        margin-bottom: 0.5rem;
                        overflow: hidden;
                        text-overflow: ellipsis;
                        white-space: nowrap;
                    ",
                    {anime.title.clone()}
                }
                
                div {
                    style: "
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                    ",
                    
                    span {
                        style: "
                            color: #a0a0b0;
                            font-size: 0.875rem;
                        ",
                        {format!("{} eps", anime.episode_count)}
                    }
                    
                    if let Some(rating) = anime.rating {
                        span {
                            style: "
                                color: #fbbf24;
                                font-size: 0.875rem;
                            ",
                            {format!("⭐ {:.1}", rating)}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AnimeGrid(anime: Vec<AnimeSummary>) -> Element {
    rsx! {
        div {
            style: "
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
                gap: 1.5rem;
            ",
            
            for item in anime {
                AnimeCard { anime: item }
            }
        }
    }
}