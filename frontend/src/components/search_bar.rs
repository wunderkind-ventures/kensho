use dioxus::prelude::*;
use crate::services::api::ApiClient;
use crate::models::AnimeSummary;

#[inline_props]
pub fn SearchBar<'a>(
    cx: Scope<'a>,
    onselect: EventHandler<'a, AnimeSummary>,
    placeholder: Option<&'a str>,
) -> Element {
    let search_query = use_state(cx, String::new);
    let search_results = use_state(cx, || Vec::<AnimeSummary>::new());
    let is_loading = use_state(cx, || false);
    let show_dropdown = use_state(cx, || false);
    let api = ApiClient::new();

    // Debounced search effect
    let search_query_clone = search_query.clone();
    let search_results_clone = search_results.clone();
    let is_loading_clone = is_loading.clone();
    let show_dropdown_clone = show_dropdown.clone();
    
    use_effect(cx, &search_query.get().clone(), |query| {
        to_owned![api, search_results_clone, is_loading_clone, show_dropdown_clone];
        async move {
            if query.len() < 2 {
                search_results_clone.set(Vec::new());
                show_dropdown_clone.set(false);
                return;
            }

            is_loading_clone.set(true);
            
            // Add a small delay for debouncing (removed for simplicity)
            
            match api.search_anime(&query).await {
                Ok(response) => {
                    search_results_clone.set(response.results);
                    show_dropdown_clone.set(!response.results.is_empty());
                }
                Err(e) => {
                    tracing::error!("Search failed: {}", e);
                    search_results_clone.set(Vec::new());
                    show_dropdown_clone.set(false);
                }
            }
            
            is_loading_clone.set(false);
        }
    });

    render! {
        div { class: "search-bar",
            style: "position: relative; width: 100%; max-width: 600px;",
            
            // Search input
            div { style: "position: relative;",
                input {
                    r#type: "text",
                    placeholder: placeholder.unwrap_or("Search anime..."),
                    value: "{search_query}",
                    oninput: move |e| {
                        search_query.set(e.value.clone());
                    },
                    onfocus: move |_| {
                        if !search_results.get().is_empty() {
                            show_dropdown.set(true);
                        }
                    },
                    style: "
                        width: 100%;
                        padding: 0.75rem 3rem 0.75rem 1rem;
                        font-size: 1rem;
                        border: 2px solid #333;
                        border-radius: 8px;
                        background: #1a1a2e;
                        color: white;
                        outline: none;
                        transition: border-color 0.2s;
                    ",
                }
                
                // Search icon or loading spinner
                div { 
                    style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%);",
                    if *is_loading.get() {
                        rsx! {
                            div { 
                                style: "
                                    width: 20px;
                                    height: 20px;
                                    border: 2px solid #666;
                                    border-top-color: #64c8ff;
                                    border-radius: 50%;
                                    animation: spin 0.8s linear infinite;
                                ",
                            }
                        }
                    } else {
                        rsx! {
                            svg {
                                width: "20",
                                height: "20",
                                view_box: "0 0 20 20",
                                fill: "none",
                                stroke: "#666",
                                stroke_width: "2",
                                path {
                                    d: "M9 17A8 8 0 1 0 9 1a8 8 0 0 0 0 16zM19 19l-4.35-4.35"
                                }
                            }
                        }
                    }
                }
            }
            
            // Dropdown results
            if *show_dropdown.get() {
                rsx! {
                    div { 
                        class: "search-dropdown",
                        style: "
                            position: absolute;
                            top: calc(100% + 0.5rem);
                            left: 0;
                            right: 0;
                            max-height: 400px;
                            overflow-y: auto;
                            background: #1a1a2e;
                            border: 1px solid #333;
                            border-radius: 8px;
                            box-shadow: 0 4px 20px rgba(0,0,0,0.3);
                            z-index: 1000;
                        ",
                        
                        // Close when clicking outside
                        onmouseleave: move |_| {
                            show_dropdown.set(false);
                        },
                        
                        for result in search_results.get().iter() {
                            SearchResult {
                                anime: result.clone(),
                                onclick: move |anime| {
                                    search_query.set(String::new());
                                    search_results.set(Vec::new());
                                    show_dropdown.set(false);
                                    onselect.call(anime);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // CSS animation for spinner
        style { 
            "@keyframes spin {{
                to {{ transform: rotate(360deg); }}
            }}"
        }
    }
}

#[inline_props]
fn SearchResult<'a>(
    cx: Scope<'a>,
    anime: AnimeSummary,
    onclick: EventHandler<'a, AnimeSummary>,
) -> Element {
    let anime_clone = anime.clone();
    
    render! {
        div {
            style: "
                display: flex;
                align-items: center;
                gap: 1rem;
                padding: 0.75rem 1rem;
                cursor: pointer;
                transition: background 0.2s;
                border-bottom: 1px solid #2a2a3e;
            ",
            onmouseover: |e| {
                if let Some(elem) = e.data.target() {
                    let _ = elem.set_attribute("style", 
                        "display: flex; align-items: center; gap: 1rem; padding: 0.75rem 1rem; cursor: pointer; transition: background 0.2s; border-bottom: 1px solid #2a2a3e; background: #2a2a3e;"
                    );
                }
            },
            onmouseout: |e| {
                if let Some(elem) = e.data.target() {
                    let _ = elem.set_attribute("style", 
                        "display: flex; align-items: center; gap: 1rem; padding: 0.75rem 1rem; cursor: pointer; transition: background 0.2s; border-bottom: 1px solid #2a2a3e;"
                    );
                }
            },
            onclick: move |_| onclick.call(anime_clone.clone()),
            
            // Poster thumbnail
            if let Some(poster_url) = &anime.poster_url {
                rsx! {
                    img {
                        src: "{poster_url}",
                        alt: "{anime.title}",
                        style: "width: 40px; height: 60px; object-fit: cover; border-radius: 4px;",
                    }
                }
            } else {
                rsx! {
                    div {
                        style: "width: 40px; height: 60px; background: #333; border-radius: 4px;",
                    }
                }
            }
            
            // Info
            div { style: "flex: 1;",
                div { 
                    style: "font-weight: 500; margin-bottom: 0.25rem;",
                    "{anime.title}"
                }
                div { 
                    style: "font-size: 0.875rem; color: #999;",
                    "{anime.episodes} episodes • {anime.status}"
                }
            }
        }
    }
}