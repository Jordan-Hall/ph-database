use crate::models::{ConvictionRecord, SearchFilters};
use serde_json;
use std::collections::HashMap;
use web_sys::{window, Storage};

pub struct Database {
    storage: Option<Storage>,
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let storage = window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| "Failed to access localStorage".to_string())?;

        Ok(Self {
            storage: Some(storage),
        })
    }

    const RECORDS_KEY: &'static str = "conviction_records";
    const COUNTER_KEY: &'static str = "record_counter";

    fn get_all_records(&self) -> Result<HashMap<String, ConvictionRecord>, String> {
        if let Some(storage) = &self.storage {
            if let Ok(Some(data)) = storage.get_item(Self::RECORDS_KEY) {
                serde_json::from_str(&data)
                    .map_err(|e| format!("Failed to parse records: {}", e))
            } else {
                Ok(HashMap::new())
            }
        } else {
            Err("Storage not available".to_string())
        }
    }

    fn save_all_records(&self, records: &HashMap<String, ConvictionRecord>) -> Result<(), String> {
        if let Some(storage) = &self.storage {
            let data = serde_json::to_string(records)
                .map_err(|e| format!("Failed to serialize records: {}", e))?;
            storage
                .set_item(Self::RECORDS_KEY, &data)
                .map_err(|_| "Failed to save records".to_string())?;
            Ok(())
        } else {
            Err("Storage not available".to_string())
        }
    }

    fn get_next_id(&self) -> Result<String, String> {
        if let Some(storage) = &self.storage {
            let counter: u32 = storage
                .get_item(Self::COUNTER_KEY)
                .ok()
                .flatten()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let next = counter + 1;
            storage
                .set_item(Self::COUNTER_KEY, &next.to_string())
                .map_err(|_| "Failed to update counter".to_string())?;

            Ok(format!("record:{}", counter))
        } else {
            Err("Storage not available".to_string())
        }
    }

    pub async fn create_record(&self, mut record: ConvictionRecord) -> Result<ConvictionRecord, String> {
        let id = self.get_next_id()?;
        record.id = Some(id.clone());

        let mut records = self.get_all_records()?;
        records.insert(id, record.clone());
        self.save_all_records(&records)?;

        Ok(record)
    }

    pub async fn get_record(&self, id: &str) -> Result<Option<ConvictionRecord>, String> {
        let records = self.get_all_records()?;
        Ok(records.get(id).cloned())
    }

    pub async fn update_record(&self, record: ConvictionRecord) -> Result<ConvictionRecord, String> {
        let id = record.id.as_ref().ok_or("Record has no ID".to_string())?;

        let mut records = self.get_all_records()?;
        records.insert(id.clone(), record.clone());
        self.save_all_records(&records)?;

        Ok(record)
    }

    pub async fn delete_record(&self, id: &str) -> Result<(), String> {
        let mut records = self.get_all_records()?;
        records.remove(id);
        self.save_all_records(&records)?;
        Ok(())
    }

    pub async fn search_records(&self, filters: SearchFilters) -> Result<Vec<ConvictionRecord>, String> {
        let records = self.get_all_records()?;

        let mut results: Vec<ConvictionRecord> = records
            .values()
            .filter(|record| {
                let mut matches = true;

                if let Some(name) = &filters.name {
                    if !name.is_empty() {
                        matches = matches && record.full_name.to_lowercase().contains(&name.to_lowercase());
                    }
                }

                if let Some(offense) = &filters.offense_type {
                    if !offense.is_empty() {
                        matches = matches && record.offense_type.to_lowercase().contains(&offense.to_lowercase());
                    }
                }

                if let Some(city) = &filters.city {
                    if !city.is_empty() {
                        matches = matches && record.city.to_lowercase().contains(&city.to_lowercase());
                    }
                }

                if let Some(court) = &filters.court_name {
                    if !court.is_empty() {
                        matches = matches && record.court_name.to_lowercase().contains(&court.to_lowercase());
                    }
                }

                matches
            })
            .cloned()
            .collect();

        // Sort by date (most recent first)
        results.sort_by(|a, b| b.conviction_date.cmp(&a.conviction_date));

        Ok(results)
    }

    pub async fn get_all(&self) -> Result<Vec<ConvictionRecord>, String> {
        let records = self.get_all_records()?;
        let mut results: Vec<ConvictionRecord> = records.values().cloned().collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }
}
