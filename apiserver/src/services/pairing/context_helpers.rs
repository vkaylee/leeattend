use crate::models::TimeEntry;
use chrono::{Duration, NaiveDate, Timelike};
use chrono_tz::Tz;
use uuid::Uuid;

use super::types::OVERNIGHT_REPROCESS_CUTOFF_HOUR;

/// Extract consumed log IDs from pre-fetched adjacent entries.
/// Shared between single-day reprocessing and range reprocessing.
pub(crate) fn extract_consumed_ids(
    entries: &[TimeEntry],
    current_date: NaiveDate,
    tz: Tz,
) -> std::collections::HashSet<Uuid> {
    let mut consumed = std::collections::HashSet::new();
    for entry in entries {
        let entry_local = entry.punch_in_time().with_timezone(&tz);
        if entry_local.date_naive() == current_date {
            continue;
        }
        if entry_local.date_naive() == current_date + Duration::days(1)
            && entry_local.hour() < OVERNIGHT_REPROCESS_CUTOFF_HOUR
        {
            continue;
        }
        if let Some(metadata) = entry.generation_metadata() {
            if let Ok(paired_meta) =
                serde_json::from_value::<super::types::PairingMetadata>(metadata.clone())
            {
                for id in paired_meta.log_ids() {
                    consumed.insert(*id);
                }
            }
        }
    }
    consumed
}

/// Extend a consumed-ID set from accumulated time entries (e.g. previously
/// processed days in a batch). Centralizes the metadata-extraction logic
/// that was previously duplicated across service_propagation / service_range.
pub(crate) fn extend_consumed_from_entries(
    consumed: &mut std::collections::HashSet<Uuid>,
    entries: &[TimeEntry],
) {
    for entry in entries {
        if let Some(metadata) = entry.generation_metadata() {
            if let Ok(paired_meta) =
                serde_json::from_value::<super::types::PairingMetadata>(metadata.clone())
            {
                for id in paired_meta.log_ids() {
                    consumed.insert(*id);
                }
            }
        }
    }
}
