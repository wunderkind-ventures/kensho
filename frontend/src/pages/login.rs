use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::services::auth::AuthState;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);
    let mut use_mock = use_signal(|| false);
    let nav = navigator();
    
    let mut auth_state = use_context::<Signal<AuthState>>();
    
    // Form validation
    let is_valid = !username.read().is_empty() && !password.read().is_empty();
    
    let handle_submit = move |_e: Event<FormData>| {
        if !is_valid {
            error.set(Some("Please enter both username and password".to_string()));
            return;
        }
        
        is_loading.set(true);
        error.set(None);
        
        let username_val = username.read().clone();
        let password_val = password.read().clone();
        let use_mock_val = *use_mock.read();
        
        spawn(async move {
            // Simulate API call
            gloo_timers::future::TimeoutFuture::new(1000).await;
            
            // In production, this would call the actual API
            if use_mock_val || (username_val == "demo" && password_val == "demo123") {
                // Set auth state
                auth_state.write().login(
                    "mock_token_123".to_string(),
                    "mock_refresh_token".to_string(),
                );
                
                // Navigate to home
                nav.push("/");
            } else {
                error.set(Some("Invalid credentials. Try 'demo' / 'demo123' or use mock login.".to_string()));
                is_loading.set(false);
            }
        });
    };
    
    let fill_mock_credentials = move |_| {
        username.set("demo".to_string());
        password.set("demo123".to_string());
        use_mock.set(true);
    };
    
    rsx! {
        div { class: "login-page",
            style: "
                min-height: 100vh;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 2rem;
            ",
            
            div { class: "login-container",
                style: "
                    background: rgba(26, 26, 46, 0.95);
                    backdrop-filter: blur(10px);
                    border-radius: 20px;
                    padding: 3rem;
                    width: 100%;
                    max-width: 400px;
                    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                ",
                
                // Header
                div { style: "text-align: center; margin-bottom: 2rem;",
                    h1 { 
                        style: "
                            font-size: 2rem;
                            font-weight: bold;
                            color: white;
                            margin-bottom: 0.5rem;
                        ",
                        "見 Kenshō"
                    }
                    p { 
                        style: "color: #a0a0b0;",
                        "Sign in to continue"
                    }
                }
                
                // Error message
                if let Some(err) = error.read().as_ref() {
                    div {
                        style: "
                            background: rgba(239, 68, 68, 0.1);
                            border: 1px solid rgba(239, 68, 68, 0.3);
                            color: #ff6464;
                            padding: 1rem;
                            border-radius: 8px;
                            margin-bottom: 1rem;
                        ",
                        {err.clone()}
                    }
                }
                
                // Login form
                form {
                    onsubmit: handle_submit,
                    
                    div { style: "margin-bottom: 1.5rem;",
                        label {
                            r#for: "username",
                            style: "
                                display: block;
                                color: #a0a0b0;
                                margin-bottom: 0.5rem;
                                font-size: 0.875rem;
                            ",
                            "Username or Email"
                        }
                        input {
                            r#type: "text",
                            id: "username",
                            value: {username.read().clone()},
                            oninput: move |e| username.set(e.value()),
                            style: "
                                width: 100%;
                                padding: 0.75rem;
                                background: rgba(255, 255, 255, 0.05);
                                border: 1px solid rgba(255, 255, 255, 0.1);
                                border-radius: 8px;
                                color: white;
                                font-size: 1rem;
                            ",
                            placeholder: "Enter your username",
                        }
                    }
                    
                    div { style: "margin-bottom: 1.5rem;",
                        label {
                            r#for: "password",
                            style: "
                                display: block;
                                color: #a0a0b0;
                                margin-bottom: 0.5rem;
                                font-size: 0.875rem;
                            ",
                            "Password"
                        }
                        input {
                            r#type: "password",
                            id: "password",
                            value: {password.read().clone()},
                            oninput: move |e| password.set(e.value()),
                            style: "
                                width: 100%;
                                padding: 0.75rem;
                                background: rgba(255, 255, 255, 0.05);
                                border: 1px solid rgba(255, 255, 255, 0.1);
                                border-radius: 8px;
                                color: white;
                                font-size: 1rem;
                            ",
                            placeholder: "Enter your password",
                        }
                    }
                    
                    button {
                        r#type: "submit",
                        disabled: !is_valid || *is_loading.read(),
                        style: {
                            format!(
                                "width: 100%; padding: 1rem; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; border-radius: 8px; font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.3s; opacity: {};",
                                if is_valid { "1.0" } else { "0.6" }
                            )
                        },
                        if *is_loading.read() {
                            "Signing in..."
                        } else {
                            "Sign In"
                        }
                    }
                }
                
                // Divider
                div {
                    style: "
                        text-align: center;
                        margin: 2rem 0;
                        position: relative;
                    ",
                    span {
                        style: "
                            background: rgba(26, 26, 46, 0.95);
                            padding: 0 1rem;
                            color: #a0a0b0;
                            position: relative;
                            z-index: 1;
                        ",
                        "OR"
                    }
                    div {
                        style: "
                            position: absolute;
                            top: 50%;
                            left: 0;
                            right: 0;
                            height: 1px;
                            background: rgba(255, 255, 255, 0.1);
                        ",
                    }
                }
                
                // Mock login button
                button {
                    onclick: fill_mock_credentials,
                    style: "
                        width: 100%;
                        padding: 1rem;
                        background: rgba(255, 255, 255, 0.05);
                        color: #a0a0b0;
                        border: 1px solid rgba(255, 255, 255, 0.1);
                        border-radius: 8px;
                        font-size: 0.875rem;
                        cursor: pointer;
                        transition: all 0.3s;
                    ",
                    "Use Mock Credentials"
                }
                
                // Back to home link
                div { style: "text-align: center; margin-top: 2rem;",
                    Link {
                        to: "/",
                        style: "
                            color: #667eea;
                            text-decoration: none;
                            font-size: 0.875rem;
                        ",
                        "← Back to Home"
                    }
                }
            }
        }
    }
}