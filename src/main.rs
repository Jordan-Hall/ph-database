mod components;
mod database;
mod models;
mod pages;

use dioxus::prelude::*;
use dioxus_router::{Router, Routable};
use components::Header;

fn main() {
    // Initialize logging
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    dioxus::launch(App);
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    HomePage {},
    #[route("/add")]
    AddRecordPage {},
    #[route("/view/:id")]
    ViewRecordPage { id: String },
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn HomePage() -> Element {
    rsx! {
        Header {}
        pages::Home {}
    }
}

#[component]
fn AddRecordPage() -> Element {
    rsx! {
        Header {}
        pages::AddRecord {}
    }
}

#[component]
fn ViewRecordPage(id: String) -> Element {
    rsx! {
        Header {}
        pages::ViewRecord { id }
    }
}
