use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConvictionRecord {
    pub id: Option<String>,
    pub full_name: String,
    pub offense_type: String,
    pub conviction_date: String,
    pub court_name: String,
    pub sentence: String,
    pub street_name: String,
    pub city: String,
    pub postcode_district: String, // First part of postcode only (e.g., "SW1A")
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub source: RecordSource,
    pub interview_consent: bool,
    pub interview_date: Option<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordSource {
    CourtRecord,
    Interview,
    Both,
}

impl ConvictionRecord {
    pub fn new() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: None,
            full_name: String::new(),
            offense_type: String::new(),
            conviction_date: String::new(),
            court_name: String::new(),
            sentence: String::new(),
            street_name: String::new(),
            city: String::new(),
            postcode_district: String::new(),
            latitude: None,
            longitude: None,
            source: RecordSource::CourtRecord,
            interview_consent: false,
            interview_date: None,
            notes: String::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub name: Option<String>,
    pub offense_type: Option<String>,
    pub city: Option<String>,
    pub court_name: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            name: None,
            offense_type: None,
            city: None,
            court_name: None,
            date_from: None,
            date_to: None,
        }
    }
}
