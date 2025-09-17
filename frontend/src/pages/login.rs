use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::services::auth::use_login_service;

#[component]
pub fn Login(cx: Scope) -> Element {
    let username = use_state(cx, String::new);
    let password = use_state(cx, String::new);
    let error = use_state(cx, || None::<String>);
    let is_loading = use_state(cx, || false);
    let use_mock = use_state(cx, || false);
    let login_service = use_login_service(cx);
    
    // Form validation
    let is_valid = !username.get().is_empty() && !password.get().is_empty();
    
    let handle_submit = move |e: FormEvent| {
        e.prevent_default();
        
        if !is_valid {
            error.set(Some("Please enter both username and password".to_string()));
            return;
        }
        
        let username_val = if *use_mock.get() {
            "mock_user".to_string()
        } else {
            username.get().clone()
        };
        
        let password_val = if *use_mock.get() {
            "mock_pass".to_string()
        } else {
            password.get().clone()
        };
        
        is_loading.set(true);
        error.set(None);
        
        cx.spawn({
            to_owned![login_service, error, is_loading];
            async move {
                match login_service.login(username_val, password_val).await {
                    Ok(_) => {
                        // Router will redirect to home
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                is_loading.set(false);
            }
        });
    };
    
    render! {
        div { class: "login-page",
            style: "
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 100%);
                padding: 1rem;
            ",
            
            div { class: "login-container",
                style: "
                    width: 100%;
                    max-width: 400px;
                    background: #1a1a2e;
                    border-radius: 16px;
                    padding: 2rem;
                    box-shadow: 0 10px 40px rgba(0,0,0,0.4);
                ",
                
                // Header
                div { 
                    style: "text-align: center; margin-bottom: 2rem;",
                    h1 { 
                        style: "
                            font-size: 2rem;
                            font-weight: 700;
                            background: linear-gradient(135deg, #64c8ff 0%, #a855f7 100%);
                            -webkit-background-clip: text;
                            -webkit-text-fill-color: transparent;
                            margin-bottom: 0.5rem;
                        ",
                        "Welcome Back"
                    }
                    p { 
                        style: "color: #999;",
                        "Sign in to access premium features"
                    }
                }
                
                // Error message
                if let Some(error_msg) = error.get() {
                    rsx! {
                        div {
                            style: "
                                background: rgba(239, 68, 68, 0.1);
                                border: 1px solid rgba(239, 68, 68, 0.3);
                                color: #ff6464;
                                padding: 0.75rem;
                                border-radius: 8px;
                                margin-bottom: 1rem;
                                text-align: center;
                            ",
                            "{error_msg}"
                        }
                    }
                }
                
                // Login form
                form {
                    onsubmit: handle_submit,
                    
                    // Username field
                    div { 
                        style: "margin-bottom: 1.5rem;",
                        label { 
                            style: "display: block; margin-bottom: 0.5rem; color: #ccc; font-size: 0.875rem;",
                            "Username"
                        }
                        input {
                            r#type: "text",
                            placeholder: "Enter your username",
                            value: "{username}",
                            oninput: move |e| username.set(e.value.clone()),
                            disabled: "{use_mock}",
                            style: "width: 100%; padding: 0.75rem; background: #0a0a0a; border: 1px solid #333; border-radius: 8px; color: white; font-size: 1rem; outline: none; transition: border-color 0.2s;",
                        }
                    }
                    
                    // Password field
                    div { 
                        style: "margin-bottom: 1rem;",
                        label { 
                            style: "display: block; margin-bottom: 0.5rem; color: #ccc; font-size: 0.875rem;",
                            "Password"
                        }
                        input {
                            r#type: "password",
                            placeholder: "Enter your password",
                            value: "{password}",
                            oninput: move |e| password.set(e.value.clone()),
                            disabled: "{use_mock}",
                            style: "width: 100%; padding: 0.75rem; background: #0a0a0a; border: 1px solid #333; border-radius: 8px; color: white; font-size: 1rem; outline: none; transition: border-color 0.2s;",
                        }
                    }
                    
                    // Mock credentials checkbox
                    div { 
                        style: "margin-bottom: 1.5rem;",
                        label { 
                            style: "display: flex; align-items: center; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: "{use_mock}",
                                onchange: move |e| {
                                    use_mock.set(e.value.parse().unwrap_or(false));
                                    if e.value.parse().unwrap_or(false) {
                                        username.set("mock_user".to_string());
                                        password.set("mock_pass".to_string());
                                    } else {
                                        username.set(String::new());
                                        password.set(String::new());
                                    }
                                },
                                style: "margin-right: 0.5rem;",
                            }
                            span { 
                                style: "color: #999; font-size: 0.875rem;",
                                "Use mock credentials (for testing)"
                            }
                        }
                    }
                    
                    // Submit button
                    button {
                        r#type: "submit",
                        disabled: "{is_loading}",
                        style: "width: 100%; padding: 0.875rem; background: linear-gradient(135deg, #64c8ff 0%, #a855f7 100%); border: none; border-radius: 8px; color: white; font-size: 1rem; font-weight: 600; cursor: pointer; transition: opacity 0.2s;",
                        
                        if *is_loading.get() {
                            rsx! {
                                span { 
                                    style: "display: flex; align-items: center; justify-content: center;",
                                    span {
                                        style: "
                                            width: 16px;
                                            height: 16px;
                                            border: 2px solid rgba(255,255,255,0.3);
                                            border-top-color: white;
                                            border-radius: 50%;
                                            animation: spin 0.8s linear infinite;
                                            margin-right: 0.5rem;
                                        ",
                                    }
                                    "Signing in..."
                                }
                            }
                        } else {
                            rsx! { "Sign In" }
                        }
                    }
                }
                
                // Additional info
                div { 
                    style: "margin-top: 1.5rem; padding-top: 1.5rem; border-top: 1px solid #333; text-align: center;",
                    p { 
                        style: "color: #666; font-size: 0.875rem;",
                        "Sign in with your Crunchyroll account or use mock credentials for testing"
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