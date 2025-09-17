use dioxus::prelude::*;

fn main() {
    // Initialize panic hook
    console_error_panic_hook::set_once();
    
    // Launch the app
    launch(app);
}

fn app() -> Element {
    rsx! {
        div {
            style: "min-height: 100vh; background: #0a0a0a; color: white; display: flex; align-items: center; justify-content: center; flex-direction: column;",
            
            h1 {
                style: "font-size: 3rem; margin-bottom: 1rem; background: linear-gradient(135deg, #64c8ff 0%, #a855f7 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent;",
                "Project Kenshō"
            }
            
            p {
                style: "color: #999; font-size: 1.2rem; margin-bottom: 1rem;",
                "Frontend successfully updated to Dioxus 0.5!"
            }
            
            p {
                style: "color: #666; font-size: 1rem;",
                "All dependencies have been updated to their latest versions."
            }
            
            div {
                style: "margin-top: 2rem;",
                a {
                    href: "http://localhost:3000/api/health",
                    style: "color: #64c8ff; text-decoration: none; margin-right: 2rem;",
                    "Test Backend Health"
                }
                a {
                    href: "http://localhost:3000/api/health/ready",
                    style: "color: #a855f7; text-decoration: none;",
                    "Test Backend Readiness"
                }
            }
        }
    }
}