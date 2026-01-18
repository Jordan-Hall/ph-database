use dioxus::prelude::*;
use crate::models::ConvictionRecord;
use web_sys::window;

#[component]
pub fn InteractiveMap(records: Vec<ConvictionRecord>) -> Element {
    let map_id = "conviction-map";
    let records_clone = records.clone();

    use_effect(move || {
        // Initialize map with JavaScript
        let script = format!(
            r#"
            if (typeof L !== 'undefined' && document.getElementById('{}')) {{
                // Remove existing map if any
                if (window.convictionMap) {{
                    window.convictionMap.remove();
                }}

                // Create new map centered on UK
                var map = L.map('{}').setView([54.5, -2.0], 6);
                window.convictionMap = map;

                L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
                    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
                    maxZoom: 18,
                }}).addTo(map);

                // Add markers for records with coordinates
                var records = {};
                records.forEach(function(record) {{
                    if (record.latitude && record.longitude) {{
                        var marker = L.marker([record.latitude, record.longitude]).addTo(map);
                        marker.bindPopup(
                            '<b>' + record.full_name + '</b><br>' +
                            'Offense: ' + record.offense_type + '<br>' +
                            'Date: ' + record.conviction_date + '<br>' +
                            'Location: ' + record.street_name + ', ' + record.city
                        );
                    }}
                }});
            }}
            "#,
            map_id,
            map_id,
            serde_json::to_string(&records_clone).unwrap_or_else(|_| "[]".to_string())
        );

        // Execute the script using js_sys::eval
        if let Some(_win) = window() {
            let _ = js_sys::eval(&script);
        }
    });

    rsx! {
        div { class: "map-container", id: "{map_id}",
            // Leaflet will render the map here
            if records.is_empty() {
                div {
                    style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #505a5f;",
                    "No locations to display. Add records with coordinates to see them on the map."
                }
            }
        }
    }
}
