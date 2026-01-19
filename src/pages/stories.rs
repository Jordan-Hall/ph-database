use dioxus::prelude::*;
use crate::api::{ApiClient, Story};

#[component]
pub fn Stories() -> Element {
    let mut stories = use_signal(|| Vec::<Story>::new());
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let api = ApiClient::new();
            if let Ok(data) = api.get_published_stories().await {
                stories.set(data);
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Survivor Stories" }
                p { class: "govuk-body-l", "Read stories from survivors who have chosen to share their experiences." }

                Link { to: "/stories/submit", class: "govuk-button", "Submit your story" }

                hr { class: "govuk-section-break govuk-section-break--m govuk-section-break--visible" }

                if loading() {
                    p { class: "govuk-body", "Loading stories..." }
                } else if stories().is_empty() {
                    div { class: "govuk-inset-text",
                        p { "No stories available yet. ", Link { to: "/stories/submit", "Be the first to share" } }
                    }
                } else {
                    for story in stories() {
                        div { class: "govuk-summary-card",
                            div { class: "govuk-summary-card__title-wrapper",
                                h2 { class: "govuk-summary-card__title", "{story.title}" }
                            }
                            div { class: "govuk-summary-card__content",
                                if let Some(warning) = story.trigger_warning {
                                    div { class: "govuk-warning-text",
                                        span { class: "govuk-warning-text__icon", "!" }
                                        strong { class: "govuk-warning-text__text",
                                            span { class: "govuk-warning-text__assistive", "Warning" }
                                            "Trigger warning: {warning}"
                                        }
                                    }
                                }
                                p { class: "govuk-body", "{story.content.chars().take(200).collect::<String>()}..." }
                                if let Some(author) = story.author_pseudonym {
                                    p { class: "govuk-body-s", "By {author}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
