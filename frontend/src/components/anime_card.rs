use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::models::AnimeSummary;

#[inline_props]
pub fn AnimeCard<'a>(cx: Scope<'a>, anime: &'a AnimeSummary) -> Element {
    let router = use_navigator(cx);
    let anime_id = anime.id.to_string();
    
    render! {
        div { 
            class: "anime-card",
            style: "
                position: relative;
                background: #1a1a2e;
                border-radius: 8px;
                overflow: hidden;
                cursor: pointer;
                transition: transform 0.2s, box-shadow 0.2s;
                height: 100%;
                display: flex;
                flex-direction: column;
            ",
            onmouseover: |e| {
                if let Some(elem) = e.data.target() {
                    let _ = elem.set_attribute("style", 
                        "position: relative; background: #1a1a2e; border-radius: 8px; overflow: hidden; cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; height: 100%; display: flex; flex-direction: column; transform: translateY(-4px); box-shadow: 0 8px 24px rgba(0,0,0,0.4);"
                    );
                }
            },
            onmouseout: |e| {
                if let Some(elem) = e.data.target() {
                    let _ = elem.set_attribute("style", 
                        "position: relative; background: #1a1a2e; border-radius: 8px; overflow: hidden; cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; height: 100%; display: flex; flex-direction: column;"
                    );
                }
            },
            onclick: move |_| {
                router.push(&format!("/anime/{}", anime_id));
            },
            
            // Poster image
            div { 
                style: "position: relative; padding-bottom: 150%; background: #2a2a3e;",
                if let Some(poster_url) = &anime.poster_url {
                    rsx! {
                        img {
                            src: "{poster_url}",
                            alt: "{anime.title}",
                            style: "
                                position: absolute;
                                top: 0;
                                left: 0;
                                width: 100%;
                                height: 100%;
                                object-fit: cover;
                            ",
                            onerror: |e| {
                                if let Some(elem) = e.data.target() {
                                    let _ = elem.set_attribute("style", "display: none;");
                                }
                            }
                        }
                    }
                }
                
                // Gradient overlay
                div {
                    style: "
                        position: absolute;
                        bottom: 0;
                        left: 0;
                        right: 0;
                        height: 60%;
                        background: linear-gradient(to bottom, transparent, rgba(0,0,0,0.8));
                        pointer-events: none;
                    ",
                }
                
                // Episode count badge
                div {
                    style: "
                        position: absolute;
                        top: 0.5rem;
                        right: 0.5rem;
                        background: rgba(0,0,0,0.8);
                        color: white;
                        padding: 0.25rem 0.5rem;
                        border-radius: 4px;
                        font-size: 0.875rem;
                        font-weight: 500;
                    ",
                    "{anime.episodes} eps"
                }
                
                // Status badge
                div {
                    style: "
                        position: absolute;
                        top: 0.5rem;
                        left: 0.5rem;
                        background: {get_status_color(&anime.status)};
                        color: white;
                        padding: 0.25rem 0.5rem;
                        border-radius: 4px;
                        font-size: 0.75rem;
                        font-weight: 600;
                        text-transform: uppercase;
                    ",
                    "{anime.status}"
                }
            }
            
            // Card info
            div { 
                style: "padding: 1rem; flex: 1; display: flex; flex-direction: column;",
                
                // Title
                h3 { 
                    style: "
                        font-size: 1rem;
                        font-weight: 600;
                        margin-bottom: 0.5rem;
                        overflow: hidden;
                        text-overflow: ellipsis;
                        display: -webkit-box;
                        -webkit-line-clamp: 2;
                        -webkit-box-orient: vertical;
                        line-height: 1.3;
                        min-height: 2.6rem;
                    ",
                    "{anime.title}"
                }
            }
        }
    }
}

// Grid container component for anime cards
#[inline_props]
pub fn AnimeGrid<'a>(cx: Scope<'a>, anime_list: &'a Vec<AnimeSummary>) -> Element {
    render! {
        div {
            style: "
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
                gap: 1.5rem;
                padding: 1rem;
            ",
            
            for anime in anime_list.iter() {
                AnimeCard { anime: anime }
            }
        }
    }
}

fn get_status_color(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "finished" | "completed" => "rgba(34, 197, 94, 0.9)",
        "ongoing" | "airing" => "rgba(59, 130, 246, 0.9)",
        "upcoming" | "not yet aired" => "rgba(251, 146, 60, 0.9)",
        _ => "rgba(100, 100, 100, 0.9)",
    }
}