use dioxus::prelude::*;
use dioxus_router::prelude::*;
use tracing_wasm;

mod components;
mod pages;
mod services;
mod models;

use pages::{Home, Login, Series, Browse};

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
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, || services::auth::AuthState::default());
    
    render! {
        Router::<Route> {}
    }
}