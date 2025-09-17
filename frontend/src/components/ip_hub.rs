use dioxus::prelude::*;
use crate::models::{Anime, Episode};

#[derive(Clone, PartialEq)]
pub enum IpHubTab {
    Overview,
    Episodes,
    Related,
    Details,
}

#[inline_props]
pub fn IpHub<'a>(cx: Scope<'a>, anime: &'a Anime, episodes: &'a Vec<Episode>) -> Element {
    let current_tab = use_state(cx, || IpHubTab::Overview);

    render! {
        div { class: "ip-hub",
            // Header with poster and basic info
            div { class: "ip-hub-header",
                style: "display: flex; gap: 2rem; padding: 2rem; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);",
                
                // Poster
                div { class: "poster-container",
                    style: "flex-shrink: 0;",
                    if let Some(poster_url) = &anime.poster_url {
                        rsx! {
                            img {
                                src: "{poster_url}",
                                alt: "{anime.title}",
                                style: "width: 200px; height: 300px; object-fit: cover; border-radius: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.3);"
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                style: "width: 200px; height: 300px; background: #2a2a3e; border-radius: 8px; display: flex; align-items: center; justify-content: center;",
                                "No Poster"
                            }
                        }
                    }
                }
                
                // Info
                div { class: "anime-info",
                    style: "flex: 1; color: white;",
                    h1 { 
                        style: "font-size: 2.5rem; margin-bottom: 1rem;",
                        "{anime.title}" 
                    }
                    
                    // Metadata badges
                    div { 
                        style: "display: flex; gap: 1rem; margin-bottom: 1rem; flex-wrap: wrap;",
                        span { 
                            style: "background: rgba(255,255,255,0.1); padding: 0.25rem 0.75rem; border-radius: 20px;",
                            "{anime.anime_type}" 
                        }
                        span { 
                            style: "background: rgba(255,255,255,0.1); padding: 0.25rem 0.75rem; border-radius: 20px;",
                            "{anime.status}" 
                        }
                        span { 
                            style: "background: rgba(255,255,255,0.1); padding: 0.25rem 0.75rem; border-radius: 20px;",
                            "{anime.episodes} episodes" 
                        }
                        if let Some(season) = &anime.season {
                            rsx! {
                                span { 
                                    style: "background: rgba(255,255,255,0.1); padding: 0.25rem 0.75rem; border-radius: 20px;",
                                    "{season.season} {season.year}" 
                                }
                            }
                        }
                    }
                    
                    // IMDb rating if available
                    if let Some(imdb) = &anime.imdb_data {
                        rsx! {
                            div { 
                                style: "margin: 1rem 0;",
                                div { 
                                    style: "display: flex; align-items: center; gap: 0.5rem;",
                                    span { style: "color: #f5c518; font-size: 1.5rem;", "★" }
                                    span { style: "font-size: 1.2rem;", "{imdb.rating}/10" }
                                    span { style: "opacity: 0.7;", "({imdb.votes} votes)" }
                                }
                            }
                        }
                    }
                    
                    // Tags
                    if !anime.tags.is_empty() {
                        rsx! {
                            div { 
                                style: "display: flex; gap: 0.5rem; flex-wrap: wrap; margin-top: 1rem;",
                                for tag in anime.tags.iter() {
                                    span { 
                                        style: "background: rgba(100,200,255,0.2); color: #64c8ff; padding: 0.2rem 0.6rem; border-radius: 4px; font-size: 0.9rem;",
                                        "{tag.name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Tab navigation
            div { class: "ip-hub-tabs",
                style: "display: flex; gap: 2rem; padding: 0 2rem; border-bottom: 1px solid #333; background: #1a1a2e;",
                
                TabButton { 
                    label: "Overview",
                    is_active: *current_tab.get() == IpHubTab::Overview,
                    onclick: move |_| current_tab.set(IpHubTab::Overview),
                }
                TabButton { 
                    label: "Episodes",
                    is_active: *current_tab.get() == IpHubTab::Episodes,
                    onclick: move |_| current_tab.set(IpHubTab::Episodes),
                }
                TabButton { 
                    label: "Related",
                    is_active: *current_tab.get() == IpHubTab::Related,
                    onclick: move |_| current_tab.set(IpHubTab::Related),
                }
                TabButton { 
                    label: "Details",
                    is_active: *current_tab.get() == IpHubTab::Details,
                    onclick: move |_| current_tab.set(IpHubTab::Details),
                }
            }
            
            // Tab content
            div { class: "ip-hub-content",
                style: "padding: 2rem; background: #0a0a0a; min-height: 400px;",
                
                match current_tab.get() {
                    IpHubTab::Overview => rsx! {
                        div {
                            if let Some(imdb) = &anime.imdb_data {
                                rsx! {
                                    div {
                                        h2 { style: "margin-bottom: 1rem;", "Synopsis" }
                                        p { style: "line-height: 1.6; color: #ccc;", "{imdb.plot}" }
                                    }
                                }
                            } else {
                                rsx! {
                                    p { style: "color: #666;", "No synopsis available." }
                                }
                            }
                        }
                    },
                    IpHubTab::Episodes => rsx! {
                        div {
                            h2 { style: "margin-bottom: 1rem;", "Episodes" }
                            if episodes.is_empty() {
                                rsx! {
                                    p { style: "color: #666;", "No episodes available." }
                                }
                            } else {
                                rsx! {
                                    div { 
                                        style: "display: grid; gap: 1rem;",
                                        for episode in episodes.iter() {
                                            div { 
                                                style: "background: #1a1a2e; padding: 1rem; border-radius: 8px; display: flex; justify-content: space-between; align-items: center;",
                                                div {
                                                    span { style: "font-weight: bold;", "Episode {episode.number}" }
                                                    if !episode.title.is_empty() {
                                                        rsx! {
                                                            span { style: "margin-left: 1rem; color: #ccc;", "{episode.title}" }
                                                        }
                                                    }
                                                }
                                                if let Some(duration) = episode.duration {
                                                    rsx! {
                                                        span { style: "color: #666;", "{duration} min" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    IpHubTab::Related => rsx! {
                        div {
                            h2 { style: "margin-bottom: 1rem;", "Related Anime" }
                            p { style: "color: #666;", "Related anime will be displayed here based on graph relationships." }
                        }
                    },
                    IpHubTab::Details => rsx! {
                        div {
                            h2 { style: "margin-bottom: 1rem;", "Details" }
                            div { style: "display: grid; gap: 0.5rem;",
                                DetailRow { label: "Title", value: &anime.title }
                                DetailRow { label: "Type", value: &anime.anime_type }
                                DetailRow { label: "Status", value: &anime.status }
                                DetailRow { label: "Episodes", value: &anime.episodes.to_string() }
                                
                                if !anime.synonyms.is_empty() {
                                    rsx! {
                                        div { style: "display: flex; padding: 0.5rem 0;",
                                            span { style: "font-weight: bold; width: 150px;", "Synonyms:" }
                                            span { style: "color: #ccc;", "{anime.synonyms.join(\", \")}" }
                                        }
                                    }
                                }
                                
                                if !anime.sources.is_empty() {
                                    rsx! {
                                        div { style: "display: flex; padding: 0.5rem 0;",
                                            span { style: "font-weight: bold; width: 150px;", "Sources:" }
                                            span { style: "color: #ccc;", "{anime.sources.len()} sources" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[inline_props]
fn TabButton<'a>(
    cx: Scope<'a>,
    label: &'a str,
    is_active: bool,
    onclick: EventHandler<'a, MouseEvent>,
) -> Element {
    let color = if *is_active { "white" } else { "#666" };
    let border = if *is_active { "2px solid #64c8ff" } else { "2px solid transparent" };
    
    render! {
        button {
            style: "background: none; border: none; color: {color}; padding: 1rem 0; cursor: pointer; border-bottom: {border}; transition: all 0.2s; font-size: 1rem;",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

#[inline_props]
fn DetailRow<'a>(cx: Scope<'a>, label: &'a str, value: &'a str) -> Element {
    render! {
        div { style: "display: flex; padding: 0.5rem 0;",
            span { style: "font-weight: bold; width: 150px;", "{label}:" }
            span { style: "color: #ccc;", "{value}" }
        }
    }
}