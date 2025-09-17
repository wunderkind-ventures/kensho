use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlVideoElement, window};

#[inline_props]
pub fn VideoPlayer<'a>(cx: Scope<'a>, stream_url: &'a str, poster_url: Option<&'a str>) -> Element {
    let video_ref = use_ref(cx, || None::<HtmlVideoElement>);
    let is_playing = use_state(cx, || false);
    let is_loading = use_state(cx, || true);
    let error = use_state(cx, || None::<String>);
    
    // Initialize HLS.js when component mounts
    use_effect(cx, &stream_url.to_string(), |url| {
        to_owned![video_ref, is_loading, error];
        async move {
            // Load HLS.js library dynamically
            let hls_loaded = load_hls_js().await;
            
            if !hls_loaded {
                error.set(Some("Failed to load HLS.js library".to_string()));
                is_loading.set(false);
                return;
            }
            
            // Setup video player with HLS
            if let Some(video_element) = video_ref.read().as_ref() {
                match setup_hls(video_element, &url) {
                    Ok(_) => {
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to initialize player: {:?}", e)));
                        is_loading.set(false);
                    }
                }
            }
        }
    });

    render! {
        div { class: "video-player-container",
            style: "position: relative; width: 100%; background: #000; border-radius: 8px; overflow: hidden;",
            
            // Video element
            video {
                r#ref: move |elem| {
                    if let Some(element) = elem {
                        let video_elem = element.downcast::<HtmlVideoElement>().unwrap();
                        video_ref.set(Some(video_elem));
                    }
                },
                controls: "true",
                poster: poster_url.unwrap_or(""),
                style: "width: 100%; height: auto; display: block;",
                onplay: move |_| is_playing.set(true),
                onpause: move |_| is_playing.set(false),
                onloadstart: move |_| is_loading.set(true),
                oncanplay: move |_| is_loading.set(false),
            }
            
            // Loading overlay
            if *is_loading.get() {
                rsx! {
                    div { 
                        style: "
                            position: absolute;
                            top: 0;
                            left: 0;
                            right: 0;
                            bottom: 0;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            background: rgba(0,0,0,0.7);
                        ",
                        div {
                            style: "
                                width: 50px;
                                height: 50px;
                                border: 3px solid rgba(255,255,255,0.3);
                                border-top-color: white;
                                border-radius: 50%;
                                animation: spin 1s linear infinite;
                            ",
                        }
                    }
                }
            }
            
            // Error overlay
            if let Some(error_msg) = error.get() {
                rsx! {
                    div { 
                        style: "
                            position: absolute;
                            top: 0;
                            left: 0;
                            right: 0;
                            bottom: 0;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            background: rgba(0,0,0,0.9);
                            color: #ff4444;
                        ",
                        div { style: "text-align: center;",
                            div { style: "font-size: 3rem; margin-bottom: 1rem;", "⚠" }
                            div { style: "font-size: 1.2rem;", "Video Error" }
                            div { style: "margin-top: 0.5rem; opacity: 0.8;", "{error_msg}" }
                        }
                    }
                }
            }
            
            // Custom controls overlay (optional)
            if !*is_playing.get() && !*is_loading.get() && error.get().is_none() {
                rsx! {
                    div { 
                        style: "
                            position: absolute;
                            top: 50%;
                            left: 50%;
                            transform: translate(-50%, -50%);
                            width: 80px;
                            height: 80px;
                            background: rgba(0,0,0,0.7);
                            border-radius: 50%;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            cursor: pointer;
                            pointer-events: none;
                        ",
                        svg {
                            width: "40",
                            height: "40",
                            view_box: "0 0 24 24",
                            fill: "white",
                            path {
                                d: "M8 5v14l11-7z"
                            }
                        }
                    }
                }
            }
        }
        
        style { 
            "@keyframes spin {{
                to {{ transform: rotate(360deg); }}
            }}"
        }
    }
}

// JavaScript functions for HLS.js integration
#[wasm_bindgen(inline_js = r#"
export async function load_hls_js() {
    if (window.Hls) {
        return true;
    }
    
    return new Promise((resolve) => {
        const script = document.createElement('script');
        script.src = 'https://cdn.jsdelivr.net/npm/hls.js@latest';
        script.onload = () => resolve(true);
        script.onerror = () => resolve(false);
        document.head.appendChild(script);
    });
}

export function setup_hls(video_element, stream_url) {
    if (!window.Hls) {
        throw new Error("HLS.js not loaded");
    }
    
    if (window.Hls.isSupported()) {
        const hls = new window.Hls({
            debug: false,
            enableWorker: true,
            lowLatencyMode: true,
            backBufferLength: 90
        });
        
        hls.loadSource(stream_url);
        hls.attachMedia(video_element);
        
        hls.on(window.Hls.Events.MANIFEST_PARSED, function() {
            console.log("HLS manifest loaded");
        });
        
        hls.on(window.Hls.Events.ERROR, function(event, data) {
            console.error("HLS error:", data);
            if (data.fatal) {
                switch(data.type) {
                    case window.Hls.ErrorTypes.NETWORK_ERROR:
                        console.error("Fatal network error, trying to recover");
                        hls.startLoad();
                        break;
                    case window.Hls.ErrorTypes.MEDIA_ERROR:
                        console.error("Fatal media error, trying to recover");
                        hls.recoverMediaError();
                        break;
                    default:
                        console.error("Fatal error, cannot recover");
                        hls.destroy();
                        break;
                }
            }
        });
        
        // Store hls instance on video element for cleanup
        video_element._hls = hls;
    } else if (video_element.canPlayType('application/vnd.apple.mpegurl')) {
        // Native HLS support (Safari)
        video_element.src = stream_url;
    } else {
        throw new Error("HLS is not supported in this browser");
    }
}

export function cleanup_hls(video_element) {
    if (video_element._hls) {
        video_element._hls.destroy();
        delete video_element._hls;
    }
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn load_hls_js() -> Result<bool, JsValue>;
    
    #[wasm_bindgen(catch)]
    fn setup_hls(video_element: &HtmlVideoElement, stream_url: &str) -> Result<(), JsValue>;
    
    fn cleanup_hls(video_element: &HtmlVideoElement);
}