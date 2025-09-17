use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::services::auth::AuthState;

#[component]
pub fn NavBar() -> Element {
    let auth_state = use_context::<Signal<AuthState>>();
    let nav = navigator();
    
    let is_authenticated = auth_state.read().is_authenticated();
    
    let handle_logout = move |_| {
        auth_state.write().logout();
        nav.push("/");
    };
    
    rsx! {
        nav {
            class: "navbar",
            style: "
                background: linear-gradient(180deg, rgba(0,0,0,0.9) 0%, rgba(0,0,0,0.7) 100%);
                backdrop-filter: blur(10px);
                padding: 1rem 2rem;
                position: sticky;
                top: 0;
                z-index: 1000;
                border-bottom: 1px solid rgba(138, 43, 226, 0.3);
            ",
            
            div {
                class: "navbar-container",
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    max-width: 1400px;
                    margin: 0 auto;
                ",
                
                // Logo and brand
                div {
                    class: "navbar-brand",
                    style: "display: flex; align-items: center; gap: 2rem;",
                    
                    Link {
                        to: "/",
                        style: "text-decoration: none; display: flex; align-items: center; gap: 0.5rem;",
                        
                        span {
                            style: "
                                font-size: 1.5rem;
                                font-weight: bold;
                                background: linear-gradient(45deg, #667eea 0%, #764ba2 100%);
                                -webkit-background-clip: text;
                                -webkit-text-fill-color: transparent;
                                background-clip: text;
                            ",
                            "見"
                        }
                        span {
                            style: "
                                font-size: 1.25rem;
                                font-weight: 600;
                                color: white;
                            ",
                            "Kenshō"
                        }
                    }
                    
                    // Main navigation links
                    div {
                        class: "navbar-links",
                        style: "display: flex; gap: 1.5rem;",
                        
                        Link {
                            to: "/",
                            class: "nav-link",
                            style: "
                                color: #e0e0e0;
                                text-decoration: none;
                                padding: 0.5rem 1rem;
                                border-radius: 0.5rem;
                                transition: all 0.3s;
                            ",
                            "Home"
                        }
                        
                        Link {
                            to: "/browse/2024/FALL",
                            class: "nav-link",
                            style: "
                                color: #e0e0e0;
                                text-decoration: none;
                                padding: 0.5rem 1rem;
                                border-radius: 0.5rem;
                                transition: all 0.3s;
                            ",
                            "Browse"
                        }
                        
                        if is_authenticated {
                            Link {
                                to: "/watchlist",
                                class: "nav-link",
                                style: "
                                    color: #e0e0e0;
                                    text-decoration: none;
                                    padding: 0.5rem 1rem;
                                    border-radius: 0.5rem;
                                    transition: all 0.3s;
                                ",
                                "Watchlist"
                            }
                        }
                    }
                }
                
                // Right side - search and user menu
                div {
                    class: "navbar-right",
                    style: "display: flex; align-items: center; gap: 2rem;",
                    
                    // Search button (compact)
                    Link {
                        to: "/",
                        class: "search-icon",
                        style: "
                            color: #e0e0e0;
                            padding: 0.5rem;
                            border-radius: 0.5rem;
                            transition: all 0.3s;
                            display: flex;
                            align-items: center;
                        ",
                        svg {
                            width: "20",
                            height: "20",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                d: "M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                            }
                        }
                    }
                    
                    // User menu
                    div {
                        class: "user-menu",
                        style: "display: flex; align-items: center; gap: 1rem;",
                        
                        if is_authenticated {
                            div {
                                style: "display: flex; align-items: center; gap: 1rem;",
                                
                                // User avatar
                                div {
                                    style: "
                                        width: 32px;
                                        height: 32px;
                                        border-radius: 50%;
                                        background: linear-gradient(45deg, #667eea 0%, #764ba2 100%);
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                        color: white;
                                        font-weight: bold;
                                    ",
                                    "U"
                                }
                                
                                button {
                                    onclick: handle_logout,
                                    style: "
                                        background: transparent;
                                        border: 1px solid rgba(138, 43, 226, 0.5);
                                        color: #e0e0e0;
                                        padding: 0.5rem 1rem;
                                        border-radius: 0.5rem;
                                        cursor: pointer;
                                        transition: all 0.3s;
                                    ",
                                    "Logout"
                                }
                            }
                        } else {
                            Link {
                                to: "/login",
                                style: "
                                    background: linear-gradient(45deg, #667eea 0%, #764ba2 100%);
                                    color: white;
                                    padding: 0.5rem 1.5rem;
                                    border-radius: 0.5rem;
                                    text-decoration: none;
                                    transition: all 0.3s;
                                ",
                                "Login"
                            }
                        }
                    }
                }
            }
        }
    }
}

// Mobile-responsive navbar with hamburger menu
#[component]
pub fn MobileNavBar() -> Element {
    let mut menu_open = use_signal(|| false);
    let auth_state = use_context::<Signal<AuthState>>();
    let nav = navigator();
    
    let toggle_menu = move |_| {
        menu_open.set(!*menu_open.read());
    };
    
    let handle_logout = move |_| {
        auth_state.write().logout();
        menu_open.set(false);
        nav.push("/");
    };
    
    rsx! {
        nav {
            class: "mobile-navbar",
            style: "
                background: rgba(0,0,0,0.95);
                padding: 1rem;
                position: sticky;
                top: 0;
                z-index: 1000;
                display: none;
            ",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                
                // Logo
                Link {
                    to: "/",
                    style: "text-decoration: none; display: flex; align-items: center; gap: 0.5rem;",
                    onclick: move |_| menu_open.set(false),
                    
                    span {
                        style: "
                            font-size: 1.25rem;
                            font-weight: bold;
                            background: linear-gradient(45deg, #667eea 0%, #764ba2 100%);
                            -webkit-background-clip: text;
                            -webkit-text-fill-color: transparent;
                            background-clip: text;
                        ",
                        "見"
                    }
                    span {
                        style: "color: white; font-weight: 600;",
                        "Kenshō"
                    }
                }
                
                // Hamburger button
                button {
                    onclick: toggle_menu,
                    class: "hamburger-menu",
                    style: "
                        background: transparent;
                        border: none;
                        color: white;
                        font-size: 1.5rem;
                        cursor: pointer;
                        padding: 0.5rem;
                    ",
                    if *menu_open.read() { "✕" } else { "☰" }
                }
            }
            
            // Mobile menu dropdown
            if *menu_open.read() {
                div {
                    class: "mobile-menu open",
                    style: "
                        background: rgba(0,0,0,0.98);
                        position: absolute;
                        top: 100%;
                        left: 0;
                        right: 0;
                        padding: 1rem;
                        display: flex;
                        flex-direction: column;
                        gap: 1rem;
                        border-top: 1px solid rgba(138, 43, 226, 0.3);
                    ",
                    
                    Link {
                        to: "/",
                        onclick: move |_| menu_open.set(false),
                        style: "
                            color: white;
                            text-decoration: none;
                            padding: 1rem;
                            border-radius: 0.5rem;
                        ",
                        "Home"
                    }
                    
                    Link {
                        to: "/browse/2024/FALL",
                        onclick: move |_| menu_open.set(false),
                        style: "
                            color: white;
                            text-decoration: none;
                            padding: 1rem;
                            border-radius: 0.5rem;
                        ",
                        "Browse"
                    }
                    
                    if auth_state.read().is_authenticated() {
                        Link {
                            to: "/watchlist",
                            onclick: move |_| menu_open.set(false),
                            style: "
                                color: white;
                                text-decoration: none;
                                padding: 1rem;
                                border-radius: 0.5rem;
                            ",
                            "Watchlist"
                        }
                        
                        button {
                            onclick: handle_logout,
                            style: "
                                background: transparent;
                                border: 1px solid rgba(138, 43, 226, 0.5);
                                color: white;
                                padding: 1rem;
                                border-radius: 0.5rem;
                                text-align: left;
                            ",
                            "Logout"
                        }
                    } else {
                        Link {
                            to: "/login",
                            onclick: move |_| menu_open.set(false),
                            style: "
                                background: linear-gradient(45deg, #667eea 0%, #764ba2 100%);
                                color: white;
                                padding: 1rem;
                                border-radius: 0.5rem;
                                text-decoration: none;
                                text-align: center;
                            ",
                            "Login"
                        }
                    }
                }
            }
        }
    }
}