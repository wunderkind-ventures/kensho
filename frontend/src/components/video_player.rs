use dioxus::prelude::*;

#[component]
pub fn VideoPlayer(stream_url: String) -> Element {
    let mut is_loading = use_signal(|| true);
    let mut has_error = use_signal(|| false);
    
    use_effect(move || {
        // In production, this would initialize HLS.js or native video player
        spawn(async move {
            // Simulate loading
            gloo_timers::future::TimeoutFuture::new(1000).await;
            is_loading.set(false);
        });
    });
    
    rsx! {
        div { class: "video-player",
            style: "
                position: relative;
                width: 100%;
                aspect-ratio: 16/9;
                background: #000;
                border-radius: 12px;
                overflow: hidden;
            ",
            
            if *is_loading.read() {
                div {
                    style: "
                        position: absolute;
                        inset: 0;
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        background: rgba(0,0,0,0.8);
                    ",
                    
                    div {
                        style: "
                            width: 60px;
                            height: 60px;
                            border: 3px solid rgba(255,255,255,0.3);
                            border-radius: 50%;
                            border-top-color: #667eea;
                            animation: spin 1s linear infinite;
                        ",
                    }
                    
                    p {
                        style: "
                            color: white;
                            margin-top: 1rem;
                            font-size: 0.9rem;
                        ",
                        "Loading stream..."
                    }
                }
            } else if *has_error.read() {
                div {
                    style: "
                        position: absolute;
                        inset: 0;
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        background: rgba(0,0,0,0.9);
                    ",
                    
                    svg {
                        width: "60",
                        height: "60",
                        fill: "#ef4444",
                        view_box: "0 0 20 20",
                        path {
                            d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                        }
                    }
                    
                    p {
                        style: "
                            color: white;
                            margin-top: 1rem;
                            font-size: 1rem;
                        ",
                        "Failed to load stream"
                    }
                    
                    button {
                        onclick: move |_| {
                            has_error.set(false);
                            is_loading.set(true);
                        },
                        style: "
                            margin-top: 1rem;
                            padding: 0.5rem 1.5rem;
                            background: #667eea;
                            color: white;
                            border: none;
                            border-radius: 8px;
                            cursor: pointer;
                        ",
                        "Retry"
                    }
                }
            } else {
                video {
                    src: {stream_url.clone()},
                    controls: true,
                    autoplay: true,
                    style: "
                        width: 100%;
                        height: 100%;
                    ",
                }
                
                // Custom controls overlay (simplified)
                div {
                    style: "
                        position: absolute;
                        bottom: 0;
                        left: 0;
                        right: 0;
                        background: linear-gradient(to top, rgba(0,0,0,0.8), transparent);
                        padding: 1rem;
                        opacity: 0.8;
                        transition: opacity 0.3s;
                    ",
                    
                    div {
                        style: "
                            display: flex;
                            justify-content: space-between;
                            align-items: center;
                        ",
                        
                        button {
                            style: "
                                background: transparent;
                                border: none;
                                color: white;
                                cursor: pointer;
                                padding: 0.5rem;
                            ",
                            "▶️"
                        }
                        
                        div {
                            style: "
                                flex: 1;
                                height: 4px;
                                background: rgba(255,255,255,0.3);
                                margin: 0 1rem;
                                border-radius: 2px;
                            ",
                            
                            div {
                                style: "
                                    height: 100%;
                                    width: 30%;
                                    background: #667eea;
                                    border-radius: 2px;
                                ",
                            }
                        }
                        
                        button {
                            style: "
                                background: transparent;
                                border: none;
                                color: white;
                                cursor: pointer;
                                padding: 0.5rem;
                            ",
                            "⛶"
                        }
                    }
                }
            }
        }
    }
}