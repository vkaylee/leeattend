use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::TimeEntry;
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

const OVERNIGHT_REPROCESS_CUTOFF_HOUR: u32 = 12;
const ADJACENT_ENTRY_SEARCH_DAYS: i64 = 2;

pub(crate) struct ConsumedLogTracker<'a> {
    time_entry_repo: &'a dyn crate::repositories::TimeEntryRepository,
    auth_token: &'a TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
    tz: Tz,
}

impl<'a> ConsumedLogTracker<'a> {
    pub(crate) fn new(
        time_entry_repo: &'a dyn crate::repositories::TimeEntryRepository,
        auth_token: &'a TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Self {
        Self {
            time_entry_repo,
            auth_token,
            employment_id,
            date,
            tz,
        }
    }

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
                if let Ok(paired_meta) = serde_json::from_value::<
                    crate::services::pairing_algo::PairingMetadata,
                >(metadata.clone())
                {
                    for id in paired_meta.log_ids() {
                        consumed.insert(*id);
                    }
                }
            }
        }
        consumed
    }

    pub(crate) async fn get_consumed_log_ids(
        &self,
        start_of_day_utc: DateTime<Utc>,
        next_day_utc: DateTime<Utc>,
    ) -> Result<std::collections::HashSet<Uuid>, AppError> {
        // Fetch adjacent time entries to check for already-paired logs
        let search_start = start_of_day_utc - Duration::days(ADJACENT_ENTRY_SEARCH_DAYS);
        let search_end = next_day_utc + Duration::days(ADJACENT_ENTRY_SEARCH_DAYS);
        let adjacent_entries = self
            .time_entry_repo
            .get_employment_time_entries(
                self.auth_token,
                self.employment_id,
                search_start,
                search_end,
            )
            .await?;

        Ok(Self::extract_consumed_ids(
            &adjacent_entries,
            self.date,
            self.tz,
        ))
    }
}
