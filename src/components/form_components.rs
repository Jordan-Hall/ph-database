use dioxus::prelude::*;

#[component]
pub fn GovukInput(
    label: String,
    hint: Option<String>,
    value: String,
    oninput: EventHandler<String>,
    input_type: Option<String>,
    width_class: Option<String>,
) -> Element {
    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let width = width_class.unwrap_or_else(|| "govuk-input--width-20".to_string());

    rsx! {
        div { class: "govuk-form-group",
            label { class: "govuk-label", "{label}" }
            if let Some(hint_text) = hint {
                div { class: "govuk-hint", "{hint_text}" }
            }
            input {
                class: "govuk-input {width}",
                r#type: "{input_type}",
                value: "{value}",
                oninput: move |evt| oninput.call(evt.value().clone()),
            }
        }
    }
}

#[component]
pub fn GovukTextarea(
    label: String,
    hint: Option<String>,
    value: String,
    oninput: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "govuk-form-group",
            label { class: "govuk-label", "{label}" }
            if let Some(hint_text) = hint {
                div { class: "govuk-hint", "{hint_text}" }
            }
            textarea {
                class: "govuk-textarea",
                value: "{value}",
                oninput: move |evt| oninput.call(evt.value().clone()),
            }
        }
    }
}

#[component]
pub fn GovukSelect(
    label: String,
    value: String,
    options: Vec<(String, String)>, // (value, display_text)
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "govuk-form-group",
            label { class: "govuk-label", "{label}" }
            select {
                class: "govuk-select",
                value: "{value}",
                onchange: move |evt| onchange.call(evt.value().clone()),
                for (opt_value, opt_text) in options {
                    option { value: "{opt_value}", "{opt_text}" }
                }
            }
        }
    }
}

#[component]
pub fn GovukCheckbox(
    label: String,
    checked: bool,
    onchange: EventHandler<bool>,
) -> Element {
    rsx! {
        div { class: "govuk-checkboxes__item",
            input {
                class: "govuk-checkboxes__input",
                r#type: "checkbox",
                checked: checked,
                onchange: move |evt| onchange.call(evt.value() == "true"),
            }
            label { class: "govuk-checkboxes__label", "{label}" }
        }
    }
}

#[component]
pub fn GovukButton(
    text: String,
    onclick: EventHandler<()>,
    button_type: Option<String>, // "primary", "secondary", "warning"
) -> Element {
    let class = match button_type.as_deref() {
        Some("secondary") => "govuk-button govuk-button--secondary",
        Some("warning") => "govuk-button govuk-button--warning",
        _ => "govuk-button",
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| onclick.call(()),
            "{text}"
        }
    }
}
