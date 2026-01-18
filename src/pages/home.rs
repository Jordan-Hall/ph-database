use dioxus::prelude::*;
use dioxus_router::Link;
use crate::components::*;
use crate::database::Database;
use crate::models::{ConvictionRecord, SearchFilters};

#[component]
pub fn Home() -> Element {
    let mut search_name = use_signal(|| String::new());
    let mut search_offense = use_signal(|| String::new());
    let mut search_city = use_signal(|| String::new());
    let mut search_court = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<ConvictionRecord>::new());
    let mut active_tab = use_signal(|| "search");

    let search_records = move |_| {
        spawn(async move {
            let db = Database::new().unwrap();
            let filters = SearchFilters {
                name: if search_name().is_empty() { None } else { Some(search_name()) },
                offense_type: if search_offense().is_empty() { None } else { Some(search_offense()) },
                city: if search_city().is_empty() { None } else { Some(search_city()) },
                court_name: if search_court().is_empty() { None } else { Some(search_court()) },
                date_from: None,
                date_to: None,
            };

            match db.search_records(filters).await {
                Ok(results) => {
                    search_results.set(results);
                }
                Err(e) => {
                    log::error!("Search error: {}", e);
                }
            }
        });
    };

    let load_all = move |_| {
        spawn(async move {
            let db = Database::new().unwrap();
            match db.get_all().await {
                Ok(results) => {
                    search_results.set(results);
                }
                Err(e) => {
                    log::error!("Load error: {}", e);
                }
            }
        });
    };

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Criminal Convictions Database" }

                div { class: "govuk-tabs",
                    ul { class: "govuk-tabs__list",
                        li { class: "govuk-tabs__list-item",
                            a {
                                class: if active_tab() == "search" { "govuk-tabs__tab govuk-tabs__tab--selected" } else { "govuk-tabs__tab" },
                                href: "#",
                                onclick: move |e| {
                                    e.prevent_default();
                                    active_tab.set("search");
                                },
                                "Search & View"
                            }
                        }
                        li { class: "govuk-tabs__list-item",
                            a {
                                class: if active_tab() == "map" { "govuk-tabs__tab govuk-tabs__tab--selected" } else { "govuk-tabs__tab" },
                                href: "#",
                                onclick: move |e| {
                                    e.prevent_default();
                                    active_tab.set("map");
                                },
                                "Map View"
                            }
                        }
                        li { class: "govuk-tabs__list-item",
                            Link {
                                to: "/add",
                                class: "govuk-tabs__tab",
                                "Add New Record"
                            }
                        }
                    }

                    if active_tab() == "search" {
                        div { class: "govuk-tabs__panel",
                            h2 { class: "govuk-heading-l", "Search Records" }

                            div { class: "govuk-grid-row",
                                div { class: "govuk-grid-column-one-half",
                                    GovukInput {
                                        label: "Name".to_string(),
                                        hint: Some("Enter full or partial name".to_string()),
                                        value: search_name(),
                                        oninput: move |val| search_name.set(val),
                                        input_type: None,
                                        width_class: Some("govuk-input--width-20".to_string()),
                                    }
                                }
                                div { class: "govuk-grid-column-one-half",
                                    GovukInput {
                                        label: "Offense Type".to_string(),
                                        hint: None,
                                        value: search_offense(),
                                        oninput: move |val| search_offense.set(val),
                                        input_type: None,
                                        width_class: Some("govuk-input--width-20".to_string()),
                                    }
                                }
                            }

                            div { class: "govuk-grid-row",
                                div { class: "govuk-grid-column-one-half",
                                    GovukInput {
                                        label: "City".to_string(),
                                        hint: None,
                                        value: search_city(),
                                        oninput: move |val| search_city.set(val),
                                        input_type: None,
                                        width_class: Some("govuk-input--width-20".to_string()),
                                    }
                                }
                                div { class: "govuk-grid-column-one-half",
                                    GovukInput {
                                        label: "Court Name".to_string(),
                                        hint: None,
                                        value: search_court(),
                                        oninput: move |val| search_court.set(val),
                                        input_type: None,
                                        width_class: Some("govuk-input--width-20".to_string()),
                                    }
                                }
                            }

                            GovukButton {
                                text: "Search".to_string(),
                                onclick: search_records,
                                button_type: None,
                            }

                            GovukButton {
                                text: "View All Records".to_string(),
                                onclick: load_all,
                                button_type: Some("secondary".to_string()),
                            }

                            div { class: "search-results",
                                h2 { class: "govuk-heading-m",
                                    "Results ({search_results().len()} records)"
                                }

                                if !search_results().is_empty() {
                                    table { class: "govuk-table",
                                        thead { class: "govuk-table__head",
                                            tr { class: "govuk-table__row",
                                                th { class: "govuk-table__header", "Name" }
                                                th { class: "govuk-table__header", "Offense" }
                                                th { class: "govuk-table__header", "Date" }
                                                th { class: "govuk-table__header", "Court" }
                                                th { class: "govuk-table__header", "Location" }
                                                th { class: "govuk-table__header", "Actions" }
                                            }
                                        }
                                        tbody { class: "govuk-table__body",
                                            for record in search_results() {
                                                tr { class: "govuk-table__row",
                                                    td { class: "govuk-table__cell", "{record.full_name}" }
                                                    td { class: "govuk-table__cell", "{record.offense_type}" }
                                                    td { class: "govuk-table__cell", "{record.conviction_date}" }
                                                    td { class: "govuk-table__cell", "{record.court_name}" }
                                                    td { class: "govuk-table__cell", "{record.street_name}, {record.city}" }
                                                    td { class: "govuk-table__cell",
                                                        Link {
                                                            to: format!("/view/{}", record.id.as_ref().unwrap_or(&"".to_string())),
                                                            class: "govuk-link",
                                                            "View"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    p { class: "govuk-body", "No records found. Try adjusting your search or add new records." }
                                }
                            }
                        }
                    }

                    if active_tab() == "map" {
                        div { class: "govuk-tabs__panel",
                            h2 { class: "govuk-heading-l", "Map View" }
                            p { class: "govuk-body",
                                "This map shows the approximate locations (street level) of recorded convictions. Click markers for details."
                            }
                            InteractiveMap { records: search_results() }
                        }
                    }
                }
            }
        }
    }
}
