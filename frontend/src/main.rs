use dioxus::prelude::*;
use dioxus_router::prelude::*;

mod components;
mod pages;
mod services;
mod models;

use pages::{Home, Login, Series, Browse};
use services::auth::AuthState;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    
    #[route("/login")]
    Login {},
    
    #[route("/anime/:id")]
    Series { id: String },
    
    #[route("/browse/:year/:season")]
    Browse { year: i32, season: String },
}

fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Initialize tracing for logging
    tracing_wasm::set_as_global_default();
    
    // Launch the Dioxus app
    launch(app);
}

fn app() -> Element {
    use_context_provider(|| Signal::new(AuthState::default()));
    
    rsx! {
        Router::<Route> {}
    }
}