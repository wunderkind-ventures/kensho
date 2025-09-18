use dioxus::prelude::*;
use crate::models::Episode;

#[component]
pub fn EpisodeList(episodes: Vec<Episode>, on_select: EventHandler<Episode>) -> Element {
    rsx! {
        div { class: "episode-list",
            style: "
                background: rgba(26, 26, 46, 0.5);
                border-radius: 12px;
                padding: 1rem;
            ",
            
            h3 {
                style: "
                    color: white;
                    font-size: 1.25rem;
                    margin-bottom: 1rem;
                ",
                "Episodes"
            }
            
            div {
                style: "
                    display: flex;
                    flex-direction: column;
                    gap: 0.5rem;
                ",
                
                for episode in episodes {
                    EpisodeItem { 
                        episode: episode,
                        on_select: move |ep| on_select.call(ep)
                    }
                }
            }
        }
    }
}

#[component]
fn EpisodeItem(episode: Episode, on_select: EventHandler<Episode>) -> Element {
    rsx! {
        button {
            onclick: move |_| on_select.call(episode.clone()),
            style: "
                display: flex;
                justify-content: space-between;
                align-items: center;
                width: 100%;
                padding: 1rem;
                background: rgba(255, 255, 255, 0.05);
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 8px;
                cursor: pointer;
                transition: all 0.3s;
                text-align: left;
            ",
            
            div {
                div {
                    style: "
                        color: white;
                        font-weight: 600;
                        margin-bottom: 0.25rem;
                    ",
                    {format!("Episode {}", episode.episode_number)}
                }
                
                if let Some(title) = &episode.title {
                    div {
                        style: "
                            color: #a0a0b0;
                            font-size: 0.875rem;
                        ",
                        {title.clone()}
                    }
                }
            }
            
            div {
                style: "
                    display: flex;
                    align-items: center;
                    gap: 1rem;
                ",
                
                span {
                    style: "
                        color: #a0a0b0;
                        font-size: 0.875rem;
                    ",
                    {format!("{} min", episode.duration_ms / 60000)}
                }
                
                span {
                    style: "
                        color: #667eea;
                        font-size: 1.25rem;
                    ",
                    "▶"
                }
            }
        }
    }
}