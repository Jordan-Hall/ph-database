use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::api::client::ApiClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub precision: String,
    pub harm_risk: String,
    pub description: String,
}

#[component]
pub fn MapView() -> Element {
    let mut map_entries = use_signal(|| Vec::<MapEntry>::new());
    let mut loading = use_signal(|| true);
    let mut error_message = use_signal(|| None::<String>);

    // Load MapLibre GL CSS
    use_effect(move || {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("Failed to get document");

        // Add MapLibre GL CSS
        if let Some(head) = document.head() {
            let link = document.create_element("link").expect("Failed to create link");
            link.set_attribute("rel", "stylesheet").ok();
            link.set_attribute("href", "https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css").ok();
            head.append_child(&link).ok();

            // Add custom map styles
            let style = document.create_element("style").expect("Failed to create style");
            style.set_text_content(Some(r#"
                #map-container {
                    width: 100%;
                    height: 600px;
                    border: 2px solid #b1b4b6;
                    border-radius: 0;
                }
                .maplibregl-popup-content {
                    font-family: "GDS Transport", arial, sans-serif;
                    padding: 15px;
                }
                .maplibregl-popup-content h3 {
                    font-size: 19px;
                    font-weight: 700;
                    margin: 0 0 10px 0;
                }
                .marker {
                    width: 24px;
                    height: 24px;
                    border-radius: 50%;
                    border: 2px solid #fff;
                    cursor: pointer;
                }
                .marker-high { background-color: #d4351c; }
                .marker-medium { background-color: #f47738; }
                .marker-low { background-color: #00703c; }
                .marker-unknown { background-color: #505a5f; }
            "#));
            head.append_child(&style).ok();
        }
    });

    // Fetch map entries from API
    use_effect(move || {
        spawn(async move {
            let client = ApiClient::new();
            match client.get::<Vec<MapEntry>>("/api/v1/map/entries").await {
                Ok(entries) => {
                    map_entries.set(entries);
                    loading.set(false);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to load map entries: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    // Initialize MapLibre GL map after entries are loaded
    use_effect(move || {
        if !loading() && error_message().is_none() {
            let entries_json = serde_json::to_string(&map_entries()).unwrap_or_else(|_| "[]".to_string());

            let script = format!(r#"
                // Wait for MapLibre GL to load
                if (typeof maplibregl === 'undefined') {{
                    const script = document.createElement('script');
                    script.src = 'https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js';
                    script.onload = function() {{ initMap(); }};
                    document.head.appendChild(script);
                }} else {{
                    initMap();
                }}

                function initMap() {{
                    // Parse map entries
                    const entries = JSON.parse('{}');

                    // Initialize map centered on UK
                    const map = new maplibregl.Map({{
                        container: 'map-container',
                        style: {{
                            version: 8,
                            sources: {{
                                'osm': {{
                                    type: 'raster',
                                    tiles: ['https://tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png'],
                                    tileSize: 256,
                                    attribution: '© OpenStreetMap contributors'
                                }}
                            }},
                            layers: [{{
                                id: 'osm-tiles',
                                type: 'raster',
                                source: 'osm',
                                minzoom: 0,
                                maxzoom: 19
                            }}]
                        }},
                        center: [-2.0, 54.5], // UK center
                        zoom: 5.5
                    }});

                    // Add navigation controls
                    map.addControl(new maplibregl.NavigationControl({{
                        showCompass: true,
                        showZoom: true,
                        visualizePitch: false
                    }}), 'top-right');

                    // Add scale control
                    map.addControl(new maplibregl.ScaleControl({{
                        maxWidth: 200,
                        unit: 'metric'
                    }}), 'bottom-left');

                    // Add markers after map loads
                    map.on('load', function() {{
                        entries.forEach(function(entry) {{
                            // Determine marker color based on harm risk
                            let markerClass = 'marker-unknown';
                            if (entry.harm_risk === 'High') markerClass = 'marker-high';
                            else if (entry.harm_risk === 'Medium') markerClass = 'marker-medium';
                            else if (entry.harm_risk === 'Low') markerClass = 'marker-low';

                            // Create marker element
                            const el = document.createElement('div');
                            el.className = 'marker ' + markerClass;
                            el.title = entry.description;

                            // Create popup with entry details
                            const popup = new maplibregl.Popup({{
                                offset: 25,
                                closeButton: true,
                                closeOnClick: false
                            }}).setHTML(`
                                <div>
                                    <h3 class="govuk-heading-s">${{entry.description}}</h3>
                                    <dl class="govuk-summary-list govuk-summary-list--no-border">
                                        <div class="govuk-summary-list__row">
                                            <dt class="govuk-summary-list__key" style="width: 40%">Risk Level</dt>
                                            <dd class="govuk-summary-list__value">${{entry.harm_risk}}</dd>
                                        </div>
                                        <div class="govuk-summary-list__row">
                                            <dt class="govuk-summary-list__key">Precision</dt>
                                            <dd class="govuk-summary-list__value">${{entry.precision}}</dd>
                                        </div>
                                        <div class="govuk-summary-list__row">
                                            <dt class="govuk-summary-list__key">Location</dt>
                                            <dd class="govuk-summary-list__value">${{entry.latitude.toFixed(4)}}, ${{entry.longitude.toFixed(4)}}</dd>
                                        </div>
                                    </dl>
                                    <a href="/report/${{entry.id}}" class="govuk-link">View full details</a>
                                </div>
                            `);

                            // Add marker to map
                            new maplibregl.Marker({{
                                element: el,
                                anchor: 'center'
                            }})
                            .setLngLat([entry.longitude, entry.latitude])
                            .setPopup(popup)
                            .addTo(map);
                        }});
                    }});
                }}
            "#, entries_json.replace('\\', "\\\\").replace('\'', "\\'"));

            js_sys::eval(&script).ok();
        }
    });

    rsx! {
        div { class: "govuk-width-container",
            div { class: "govuk-main-wrapper",
                h1 { class: "govuk-heading-xl", "Map View" }

                if let Some(error) = error_message() {
                    div { class: "govuk-error-summary",
                        h2 { class: "govuk-error-summary__title", "There is a problem" }
                        div { class: "govuk-error-summary__body",
                            p { "{error}" }
                        }
                    }
                }

                if loading() {
                    div { class: "govuk-inset-text",
                        p { "Loading map data..." }
                    }
                } else {
                    div { class: "govuk-grid-row",
                        div { class: "govuk-grid-column-full",
                            div { class: "govuk-inset-text govuk-!-margin-bottom-4",
                                p { class: "govuk-body-s",
                                    "This map displays incident locations with privacy-preserving fuzzy coordinates. "
                                    "Markers are color-coded by harm risk: "
                                    span { style: "color: #d4351c; font-weight: bold;", "High (red)" }
                                    ", "
                                    span { style: "color: #f47738; font-weight: bold;", "Medium (orange)" }
                                    ", "
                                    span { style: "color: #00703c; font-weight: bold;", "Low (green)" }
                                    ". Click markers for details."
                                }
                            }
                            div { id: "map-container" }
                            div { class: "govuk-body-s govuk-!-margin-top-2",
                                p {
                                    "Showing {map_entries().len()} location(s). Map data © "
                                    a {
                                        href: "https://www.openstreetmap.org/copyright",
                                        target: "_blank",
                                        class: "govuk-link",
                                        "OpenStreetMap contributors"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
