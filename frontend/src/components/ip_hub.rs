use dioxus::prelude::*;
use crate::models::{Anime, Episode};

#[component]
pub fn IpHub(anime: Anime, episodes: Vec<Episode>) -> Element {
    let mut active_tab = use_signal(|| "overview");
    
    rsx! {
        div { class: "ip-hub",
            style: "
                background: rgba(26, 26, 46, 0.6);
                border-radius: 12px;
                padding: 1.5rem;
            ",
            
            // Tabs
            div {
                style: "
                    display: flex;
                    gap: 1rem;
                    border-bottom: 2px solid rgba(255, 255, 255, 0.1);
                    margin-bottom: 1.5rem;
                ",
                
                TabButton { 
                    label: "Overview",
                    is_active: *active_tab.read() == "overview",
                    on_click: move |_| active_tab.set("overview")
                }
                
                TabButton { 
                    label: "Episodes",
                    is_active: *active_tab.read() == "episodes",
                    on_click: move |_| active_tab.set("episodes")
                }
                
                TabButton { 
                    label: "Details",
                    is_active: *active_tab.read() == "details",
                    on_click: move |_| active_tab.set("details")
                }
            }
            
            // Tab content
            match active_tab.read().as_ref() {
                "overview" => rsx! {
                    div {
                        h2 {
                            style: "color: white; margin-bottom: 1rem;",
                            {anime.title.clone()}
                        }
                        p {
                            style: "color: #a0a0b0; line-height: 1.6;",
                            {anime.description.clone()}
                        }
                    }
                },
                "episodes" => rsx! {
                    div {
                        style: "
                            display: grid;
                            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
                            gap: 1rem;
                        ",
                        
                        for episode in episodes.iter() {
                            div {
                                style: "
                                    background: rgba(255, 255, 255, 0.05);
                                    border-radius: 8px;
                                    padding: 1rem;
                                ",
                                
                                h4 {
                                    style: "color: white; margin-bottom: 0.5rem;",
                                    {format!("Episode {}", episode.episode_number)}
                                }
                                
                                if let Some(title) = &episode.title {
                                    p {
                                        style: "color: #a0a0b0; font-size: 0.875rem;",
                                        {title.clone()}
                                    }
                                }
                            }
                        }
                    }
                },
                "details" => rsx! {
                    div {
                        style: "
                            display: grid;
                            grid-template-columns: repeat(2, 1fr);
                            gap: 1rem;
                        ",
                        
                        DetailRow { label: "Status", value: anime.status.clone() }
                        DetailRow { label: "Type", value: anime.anime_type.clone() }
                        DetailRow { label: "Episodes", value: format!("{}", anime.episode_count) }
                        
                        if let Some(rating) = anime.rating {
                            DetailRow { label: "Rating", value: format!("{:.1}/10", rating) }
                        }
                    }
                },
                _ => rsx! { div {} }
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, is_active: bool, on_click: EventHandler<()>) -> Element {
    rsx! {
        button {
            onclick: move |_| on_click.call(()),
            style: {
                format!(
                    "background: transparent; border: none; color: {}; padding: 0.5rem 1rem; cursor: pointer; border-bottom: 2px solid {}; transition: all 0.3s;",
                    if is_active { "white" } else { "#a0a0b0" },
                    if is_active { "#667eea" } else { "transparent" }
                )
            },
            {label}
        }
    }
}

#[component]
fn DetailRow(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                justify-content: space-between;
                padding: 0.75rem;
                background: rgba(255, 255, 255, 0.05);
                border-radius: 8px;
            ",
            
            span {
                style: "color: #a0a0b0;",
                {label}
            }
            
            span {
                style: "color: white; font-weight: 500;",
                {value}
            }
        }
    }
}