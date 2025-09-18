use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::services::api::ApiClient;
use crate::models::AnimeSummary;

#[component]
pub fn SearchBar() -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(|| Vec::<AnimeSummary>::new());
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);
    let nav = navigator();
    
    let mut search = move |_| {
        let search_query = query.read().clone();
        if search_query.len() < 2 {
            results.set(Vec::new());
            show_dropdown.set(false);
            return;
        }
        
        is_searching.set(true);
        spawn(async move {
            let api = ApiClient::new();
            match api.search(&search_query).await {
                Ok(search_results) => {
                    results.set(search_results);
                    show_dropdown.set(true);
                }
                Err(e) => {
                    tracing::error!("Search failed: {}", e);
                }
            }
            is_searching.set(false);
        });
    };
    
    rsx! {
        div { class: "search-bar",
            style: "position: relative;",
            
            div {
                style: "
                    display: flex;
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 50px;
                    padding: 0.75rem 1.5rem;
                    backdrop-filter: blur(10px);
                ",
                
                input {
                    r#type: "text",
                    value: {query.read().clone()},
                    oninput: move |e| query.set(e.value()),
                    onkeyup: search,
                    onfocus: move |_| show_dropdown.set(true),
                    placeholder: "Search anime...",
                    style: "
                        flex: 1;
                        background: transparent;
                        border: none;
                        color: white;
                        font-size: 1rem;
                        outline: none;
                    ",
                }
                
                if *is_searching.read() {
                    div {
                        style: "
                            width: 20px;
                            height: 20px;
                            border: 2px solid rgba(255,255,255,0.3);
                            border-radius: 50%;
                            border-top-color: white;
                            animation: spin 1s linear infinite;
                        ",
                    }
                } else {
                    svg {
                        width: "20",
                        height: "20",
                        fill: "white",
                        view_box: "0 0 20 20",
                        path {
                            d: "M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                        }
                    }
                }
            }
            
            // Search results dropdown
            if *show_dropdown.read() && !results.read().is_empty() {
                div {
                    style: "
                        position: absolute;
                        top: 100%;
                        left: 0;
                        right: 0;
                        margin-top: 0.5rem;
                        background: rgba(26, 26, 46, 0.98);
                        border-radius: 12px;
                        box-shadow: 0 10px 30px rgba(0,0,0,0.5);
                        max-height: 400px;
                        overflow-y: auto;
                        z-index: 100;
                    ",
                    
                    for result in results.read().clone() {
                        button {
                            onclick: move |_| {
                                let _ = nav.push(format!("/anime/{}", result.id));
                                show_dropdown.set(false);
                            },
                            style: "
                                display: flex;
                                gap: 1rem;
                                padding: 1rem;
                                width: 100%;
                                text-align: left;
                                background: transparent;
                                border: none;
                                cursor: pointer;
                                transition: background 0.2s;
                            ",
                            
                            img {
                                src: {result.poster_url},
                                style: "
                                    width: 50px;
                                    height: 70px;
                                    object-fit: cover;
                                    border-radius: 4px;
                                ",
                            }
                            
                            div {
                                style: "flex: 1;",
                                h4 {
                                    style: "
                                        color: white;
                                        font-size: 0.95rem;
                                        margin-bottom: 0.25rem;
                                    ",
                                    {result.title}
                                }
                                p {
                                    style: "
                                        color: #a0a0b0;
                                        font-size: 0.85rem;
                                    ",
                                    {format!("{} Episodes", result.episode_count)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}