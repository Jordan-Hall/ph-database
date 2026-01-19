use dioxus::prelude::*;
use crate::api::{ApiClient, PublishedItem};

#[component]
pub fn PublishedItem(slug: String) -> Element {
    let mut item = use_signal(|| None::<PublishedItem>);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        let item_slug = slug.clone();
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            
            match api.get_published_item(&item_slug).await {
                Ok(data) => item.set(Some(data)),
                Err(e) => log::error!("Failed to load item: {}", e),
            }
            
            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                if loading() {
                    p { class: "govuk-body", "Loading..." }
                } else if let Some(i) = item() {
                    div {
                        h1 { class: "govuk-heading-xl", "{i.title}" }
                        div { class: "govuk-body", dangerous_inner_html: "{i.content}" }
                        hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }
                        p { class: "govuk-body-s", "Published: {i.published_at.map(|d| d.format(\"%Y-%m-%d\").to_string()).unwrap_or_default()}" }
                    }
                } else {
                    div { class: "govuk-error-summary",
                        h2 { class: "govuk-error-summary__title", "Item not found" }
                    }
                }
            }
        }
    }
}
