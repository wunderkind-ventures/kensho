use dioxus::prelude::*;
use crate::models::Episode;
use crate::services::auth::use_auth;

#[inline_props]
pub fn EpisodeList<'a>(
    cx: Scope<'a>,
    episodes: &'a Vec<Episode>,
    anime_id: &'a str,
    onplay: EventHandler<'a, (String, i32)>, // (anime_id, episode_number)
) -> Element {
    let auth = use_auth(cx);
    let is_authenticated = auth.read().is_authenticated;
    let selected_episode = use_state(cx, || None::<i32>);
    
    render! {
        div { class: "episode-list",
            style: "width: 100%;",
            
            // List header
            div { 
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 1rem;
                    background: #1a1a2e;
                    border-radius: 8px 8px 0 0;
                ",
                h2 { style: "font-size: 1.25rem; font-weight: 600;", "Episodes" }
                span { style: "color: #999;", "{episodes.len()} episodes" }
            }
            
            // Episodes container
            div {
                style: "
                    max-height: 600px;
                    overflow-y: auto;
                    background: #0a0a0a;
                    border: 1px solid #1a1a2e;
                    border-top: none;
                    border-radius: 0 0 8px 8px;
                ",
                
                if episodes.is_empty() {
                    rsx! {
                        div { 
                            style: "padding: 3rem; text-align: center; color: #666;",
                            "No episodes available"
                        }
                    }
                } else {
                    rsx! {
                        div {
                            for episode in episodes.iter() {
                                EpisodeItem {
                                    episode: episode,
                                    is_selected: selected_episode.get() == &Some(episode.number),
                                    is_authenticated: is_authenticated,
                                    onclick: move |ep_num| {
                                        selected_episode.set(Some(ep_num));
                                        if is_authenticated {
                                            onplay.call((anime_id.to_string(), ep_num));
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
fn EpisodeItem<'a>(
    cx: Scope<'a>,
    episode: &'a Episode,
    is_selected: bool,
    is_authenticated: bool,
    onclick: EventHandler<'a, i32>,
) -> Element {
    let episode_number = episode.number;
    let cursor = if *is_authenticated { "pointer" } else { "default" };
    let background = if *is_selected { "#1a1a2e" } else { "transparent" };
    
    render! {
        div {
            class: "episode-item",
            style: "display: flex; align-items: center; padding: 1rem; border-bottom: 1px solid #1a1a2e; cursor: {cursor}; background: {background}; transition: background 0.2s;",
            onmouseover: move |e| {
                if *is_authenticated {
                    if let Some(elem) = e.data.target() {
                        let bg = if *is_selected { "#2a2a3e" } else { "#111122" };
                        let _ = elem.set_attribute("style", 
                            &format!("display: flex; align-items: center; padding: 1rem; border-bottom: 1px solid #1a1a2e; cursor: pointer; background: {}; transition: background 0.2s;", bg)
                        );
                    }
                }
            },
            onmouseout: move |e| {
                if let Some(elem) = e.data.target() {
                    let bg = if *is_selected { "#1a1a2e" } else { "transparent" };
                    let cursor = if *is_authenticated { "pointer" } else { "default" };
                    let _ = elem.set_attribute("style", 
                        &format!("display: flex; align-items: center; padding: 1rem; border-bottom: 1px solid #1a1a2e; cursor: {}; background: {}; transition: background 0.2s;", cursor, bg)
                    );
                }
            },
            onclick: move |_| {
                onclick.call(episode_number);
            },
            
            // Episode number
            div {
                style: "
                    width: 50px;
                    height: 50px;
                    background: #2a2a3e;
                    border-radius: 8px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-weight: 600;
                    font-size: 1.1rem;
                    margin-right: 1rem;
                    flex-shrink: 0;
                ",
                "{episode.number}"
            }
            
            // Episode info
            div { style: "flex: 1;",
                div { 
                    style: "font-weight: 500; margin-bottom: 0.25rem;",
                    if !episode.title.is_empty() && episode.title != format!("Episode {}", episode.number) {
                        rsx! { "{episode.title}" }
                    } else {
                        rsx! { "Episode {episode.number}" }
                    }
                }
                if let Some(duration) = episode.duration {
                    rsx! {
                        div { 
                            style: "font-size: 0.875rem; color: #999;",
                            "{duration} minutes"
                        }
                    }
                }
            }
            
            // Play button
            if *is_authenticated {
                rsx! {
                    div {
                        style: "
                            width: 40px;
                            height: 40px;
                            background: rgba(100, 200, 255, 0.1);
                            border-radius: 50%;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            transition: background 0.2s;
                        ",
                        svg {
                            width: "20",
                            height: "20",
                            view_box: "0 0 24 24",
                            fill: "#64c8ff",
                            path {
                                d: "M8 5v14l11-7z"
                            }
                        }
                    }
                }
            } else {
                rsx! {
                    div {
                        style: "
                            padding: 0.25rem 0.75rem;
                            background: rgba(255, 100, 100, 0.1);
                            color: #ff6464;
                            border-radius: 20px;
                            font-size: 0.875rem;
                        ",
                        "Login required"
                    }
                }
            }
        }
    }
}