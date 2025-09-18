use dioxus::prelude::*;
use dioxus_router::prelude::*;

mod components;
mod pages;
mod models;
mod services;

use services::auth::AuthState;
use pages::Home;
use pages::Login;
use pages::Series;
use pages::Browse;

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
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
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

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div {
            h1 { "Page Not Found" }
            p { {format!("The route you requested does not exist: {:?}", route)} }
        }
    }
}