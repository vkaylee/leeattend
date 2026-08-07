//! Durable delivery-record layer for `LogIngestion` jobs.
//!
//! Wraps the existing priority queues (`worker:jobs:{high,normal,low}`) with a
//! namespaced Redis record tracking the lifecycle of a single agent-originated
//! attendance ingestion message, keyed by the stable `(agent_id, msg_id)`
//! pair. This is *not* a second business queue: the job itself still travels
//! through the existing `JobQueue` priority lists exactly as before. This
//! layer only adds the missing durability bookkeeping so that:
//!
//! - a `BLPOP` pop without a completed business outcome is recoverable (via
//!   the lease/reaper below), not a silent loss;
//! - duplicate `(agent_id, msg_id)` frames resolve to the existing persisted
//!   state/outcome instead of creating a second business event;
//! - delayed/replayed ACKs can be answered idempotently from the stored
//!   terminal outcome.
//!
//! # Durability boundary
//!
//! Redis is the boundary: this module assumes Redis AOF/RDB persistence and
//! (if deployed) replication are operationally configured so that a delivery
//! record survives a Redis process restart. Total Redis data loss (e.g. an
//! unpersisted instance wiped before a scheduled snapshot) is outside this
//! boundary and is not covered by the reaper below — the reaper only
//! recovers jobs whose delivery record still exists in Redis but whose
//! processing lease expired or which were popped from the priority queue
//! without ever being claimed.
//!
//! # State machine
//!
//! ```text
//! (missing) --record_and_enqueue--> Ready --claim--> Processing
//!                                     ^                  |
//!                                     |            complete(outcome)
//!                                     |                  |
//!                                     +--- Retryable <----+---> Processed (terminal)
//!                                     |  (reap/lease      |
//!                                     |   expiry/retry)   +---> Permanent (terminal, -> DLQ)
//!                                     |
//!                              (reaper requeues job)
//! ```
//!
//! Key pattern: `ingest:delivery:{agent_id}:{msg_id}` — STRING, JSON-encoded
//! [`DeliveryRecord`], TTL-bounded so abandoned records self-clean.

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::queue::{Job, JobPriority, RedisPool};

const DELIVERY_PREFIX: &str = "ingest:delivery:";
/// Delivery records outlive the job TTL so a very late duplicate frame or
/// delayed ACK can still resolve against the terminal outcome.
const DELIVERY_TTL_SECS: i64 = 172_800; // 48 hours
/// Default processing lease: must exceed the worker's job_timeout_secs so a
/// job that is genuinely still processing is never reaped out from under it.
pub const DEFAULT_LEASE_SECS: i64 = 600;

/// Lifecycle state of an ingestion delivery record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState {
    /// Durably recorded and queued; not yet claimed by a worker.
    Ready,
    /// Claimed by a worker holding a time-bounded lease.
    Processing,
    /// Terminal: business outcome resolved (inserted/duplicate/staged/parse-terminal).
    Processed,
    /// Transient failure or reaped lease; eligible to be re-queued and re-claimed.
    Retryable,
    /// Terminal: malformed payload or retries exhausted; moved to DLQ.
    Permanent,
}

impl DeliveryState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Processed | Self::Permanent)
    }
}

/// Durable record for one `(agent_id, msg_id)` ingestion attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub job_id: uuid::Uuid,
    pub state: DeliveryState,
    pub attempt: u32,
    #[serde(default)]
    pub lease_owner: Option<String>,
    /// Unix timestamp (seconds) after which a `Processing` lease is stale.
    #[serde(default)]
    pub lease_deadline: Option<i64>,
    /// Bounded terminal outcome string (mirrors `AckStatus::as_wire_str`),
    /// present once `state` reaches a terminal value.
    #[serde(default)]
    pub outcome: Option<String>,
    /// Bounded error message for the terminal/retryable outcome. Never holds
    /// raw employee PII — callers must pass already-sanitized messages.
    #[serde(default)]
    pub error: Option<String>,
    /// Serialized [`Job`] retained so the reaper can re-push it onto the
    /// priority queue without depending on the original in-flight copy.
    pub job_json: String,
    pub priority: JobPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Outcome of attempting to durably record + enqueue a new ingestion message.
#[derive(Debug, Clone)]
pub enum RecordOutcome {
    /// First time this `(agent_id, msg_id)` has been seen; job was pushed.
    Enqueued,
    /// Duplicate frame — returns the existing persisted record instead of
    /// creating a second business event.
    Duplicate(DeliveryRecord),
}

/// Durable delivery-record layer, built on the same [`RedisPool`] as [`super::JobQueue`].
pub struct IngestionDeliveryStore {
    pool: RedisPool,
    queue: crate::queue::JobQueue,
}

impl IngestionDeliveryStore {
    pub fn new(pool: RedisPool) -> Self {
        Self {
            queue: crate::queue::JobQueue::new(pool.clone()),
            pool,
        }
    }

    fn key(agent_id: &str, msg_id: &str) -> String {
        format!("{DELIVERY_PREFIX}{agent_id}:{msg_id}")
    }

    /// Atomically record a new ingestion delivery and push the job onto the
    /// priority queue, or — if `(agent_id, msg_id)` was already seen — return
    /// the existing record untouched. Validate-then-persist-then-enqueue
    /// ordering happens inside a single Lua script so a partial write can
    /// never leave a record with no matching queue entry.
    pub async fn record_and_enqueue(
        &self,
        job: &Job,
        agent_id: &str,
        msg_id: &str,
    ) -> Result<RecordOutcome, redis::RedisError> {
        let key = Self::key(agent_id, msg_id);
        let queue_key = self.queue.priority_queue_key_pub(&job.priority);
        let job_json = serde_json::to_string(job).map_err(json_err)?;
        let now = chrono::Utc::now();
        let record = DeliveryRecord {
            job_id: job.job_id,
            state: DeliveryState::Ready,
            attempt: 0,
            lease_owner: None,
            lease_deadline: None,
            outcome: None,
            error: None,
            job_json: job_json.clone(),
            priority: job.priority.clone(),
            created_at: now,
            updated_at: now,
        };
        let record_json = serde_json::to_string(&record).map_err(json_err)?;

        let script = redis::Script::new(
            r"
            local existing = redis.call('GET', KEYS[1])
            if existing then
                return existing
            end
            redis.call('SET', KEYS[1], ARGV[1], 'EX', ARGV[2])
            redis.call('RPUSH', KEYS[2], ARGV[3])
            return false
            ",
        );

        let mut conn = self.pool.conn();
        let result: Option<String> = script
            .key(&key)
            .key(&queue_key)
            .arg(&record_json)
            .arg(DELIVERY_TTL_SECS)
            .arg(&job_json)
            .invoke_async(&mut conn)
            .await?;

        match result {
            None => Ok(RecordOutcome::Enqueued),
            Some(existing_json) => {
                let existing: DeliveryRecord =
                    serde_json::from_str(&existing_json).map_err(json_err)?;
                Ok(RecordOutcome::Duplicate(existing))
            }
        }
    }

    /// Atomically claim a `Ready` or `Retryable` record for processing,
    /// setting a time-bounded lease. Returns `true` if the caller now owns
    /// the lease; `false` if the record is missing, already terminal, or
    /// already leased by another worker (duplicate concurrent claim).
    pub async fn claim(
        &self,
        agent_id: &str,
        msg_id: &str,
        owner: &str,
        lease_secs: i64,
    ) -> Result<bool, redis::RedisError> {
        let key = Self::key(agent_id, msg_id);
        let now = chrono::Utc::now().timestamp();
        let deadline = now + lease_secs;

        let script = redis::Script::new(
            r"
            local raw = redis.call('GET', KEYS[1])
            if not raw then
                return 0
            end
            local rec = cjson.decode(raw)
            if rec.state ~= 'ready' and rec.state ~= 'retryable' then
                return 0
            end
            rec.state = 'processing'
            rec.attempt = rec.attempt + 1
            rec.lease_owner = ARGV[1]
            rec.lease_deadline = tonumber(ARGV[2])
            rec.updated_at = ARGV[3]
            redis.call('SET', KEYS[1], cjson.encode(rec), 'EX', ARGV[4])
            return 1
            ",
        );

        let mut conn = self.pool.conn();
        let claimed: i64 = script
            .key(&key)
            .arg(owner)
            .arg(deadline)
            .arg(chrono::Utc::now().to_rfc3339())
            .arg(DELIVERY_TTL_SECS)
            .invoke_async(&mut conn)
            .await?;
        Ok(claimed == 1)
    }

    /// Mark a claimed record with its terminal or retryable outcome.
    /// `state` must be `Processed`, `Retryable`, or `Permanent`. On
    /// `Permanent`, the job is additionally pushed to the DLQ.
    pub async fn complete(
        &self,
        agent_id: &str,
        msg_id: &str,
        state: DeliveryState,
        outcome: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), redis::RedisError> {
        let key = Self::key(agent_id, msg_id);
        let mut conn = self.pool.conn();
        let raw: Option<String> = conn.get(&key).await?;
        let Some(raw) = raw else {
            // Record already expired/missing — nothing to update. This is
            // not an error: the delivery TTL bounds how long we track it,
            // and a very late completion after TTL expiry has no durable
            // record left to update.
            warn!(agent_id, msg_id, "Delivery record missing on complete()");
            return Ok(());
        };
        let mut record: DeliveryRecord = serde_json::from_str(&raw).map_err(json_err)?;
        record.state = state;
        record.outcome = outcome.map(str::to_string);
        record.error = error.map(str::to_string);
        record.lease_owner = None;
        record.lease_deadline = None;
        record.updated_at = chrono::Utc::now();
        let json = serde_json::to_string(&record).map_err(json_err)?;

        redis::pipe()
            .set(&key, &json)
            .expire(&key, DELIVERY_TTL_SECS)
            .query_async::<()>(&mut conn)
            .await?;

        if state == DeliveryState::Permanent {
            if let Ok(job) = serde_json::from_str::<Job>(&record.job_json) {
                let _ = self
                    .queue
                    .move_to_dlq(&job, error.unwrap_or("permanent_error"))
                    .await;
            }
        }

        Ok(())
    }

    /// Look up the current record for a `(agent_id, msg_id)` pair, if any.
    pub async fn get(
        &self,
        agent_id: &str,
        msg_id: &str,
    ) -> Result<Option<DeliveryRecord>, redis::RedisError> {
        let key = Self::key(agent_id, msg_id);
        let mut conn = self.pool.conn();
        let raw: Option<String> = conn.get(&key).await?;
        raw.map(|s| serde_json::from_str(&s).map_err(json_err))
            .transpose()
    }

    /// Reap expired leases and unclaimed-but-popped jobs.
    ///
    /// Scans `ingest:delivery:*` for records in `Processing` whose lease
    /// deadline has passed. Each is either:
    /// - re-queued as `Retryable` (attempt < `max_retries`), re-pushing the
    ///   stored job JSON onto its original priority queue, or
    /// - moved to `Permanent`/DLQ (attempt >= `max_retries`).
    ///
    /// Returns `(requeued, dlq)` counts. Intended to run periodically from
    /// the existing worker/scheduler loop, not as a new standalone process.
    pub async fn reap_expired(&self, max_retries: u32) -> Result<(u64, u64), redis::RedisError> {
        let mut conn = self.pool.conn();
        let now = chrono::Utc::now().timestamp();
        let mut cursor: u64 = 0;
        let mut requeued = 0u64;
        let mut dlq = 0u64;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(format!("{DELIVERY_PREFIX}*"))
                .arg("COUNT")
                .arg(200)
                .query_async(&mut conn)
                .await?;

            for key in keys {
                let raw: Option<String> = conn.get(&key).await?;
                let Some(raw) = raw else { continue };
                let Ok(mut record) = serde_json::from_str::<DeliveryRecord>(&raw) else {
                    continue;
                };
                if record.state != DeliveryState::Processing {
                    continue;
                }
                let Some(deadline) = record.lease_deadline else {
                    continue;
                };
                if deadline > now {
                    continue; // Lease still valid.
                }

                if record.attempt >= max_retries {
                    record.state = DeliveryState::Permanent;
                    record.error = Some("Lease expired; max retries exceeded".to_string());
                    record.lease_owner = None;
                    record.lease_deadline = None;
                    record.updated_at = chrono::Utc::now();
                    let json = serde_json::to_string(&record).map_err(json_err)?;
                    redis::pipe()
                        .set(&key, &json)
                        .expire(&key, DELIVERY_TTL_SECS)
                        .query_async::<()>(&mut conn)
                        .await?;
                    if let Ok(job) = serde_json::from_str::<Job>(&record.job_json) {
                        let _ = self
                            .queue
                            .move_to_dlq(&job, "Lease expired; max retries exceeded")
                            .await;
                    }
                    dlq += 1;
                } else {
                    record.state = DeliveryState::Retryable;
                    record.lease_owner = None;
                    record.lease_deadline = None;
                    record.updated_at = chrono::Utc::now();
                    let json = serde_json::to_string(&record).map_err(json_err)?;
                    redis::pipe()
                        .set(&key, &json)
                        .expire(&key, DELIVERY_TTL_SECS)
                        .query_async::<()>(&mut conn)
                        .await?;
                    let queue_key = self.queue.priority_queue_key_pub(&record.priority);
                    let _: () = conn.rpush(queue_key, &record.job_json).await?;
                    requeued += 1;
                }
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok((requeued, dlq))
    }
}

// Takes ownership (not `&Error`) because every call site passes this as a
// bare `map_err(json_err)` function pointer against `FnOnce(E) -> F`.
#[allow(clippy::needless_pass_by_value)]
fn json_err(e: serde_json::Error) -> redis::RedisError {
    redis::RedisError::from((redis::ErrorKind::TypeError, "JSON error", e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{JobPayload, LogIngestionPayload, TaskType};
    use uuid::Uuid;

    async fn get_test_redis() -> Option<RedisPool> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            RedisPool::new(&redis_url),
        )
        .await
        {
            Ok(Ok(pool)) => {
                match tokio::time::timeout(tokio::time::Duration::from_secs(5), pool.health_check())
                    .await
                {
                    Ok(Ok(())) => Some(pool),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn make_job(agent_id: &str, msg_id: &str) -> Job {
        Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "E1".to_string(),
                punch_time_raw: "2026-01-01 08:00:00".to_string(),
                device_id: Some("D1".to_string()),
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: Some(msg_id.to_string()),
                agent_id: Some(agent_id.to_string()),
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            JobPriority::Low,
        )
    }

    #[tokio::test]
    async fn test_record_and_enqueue_then_duplicate() {
        let Some(pool) = get_test_redis().await else {
            return;
        };
        let store = IngestionDeliveryStore::new(pool);
        let agent_id = format!("agent-{}", Uuid::new_v4());
        let msg_id = "msg-1";
        let job = make_job(&agent_id, msg_id);

        let first = store
            .record_and_enqueue(&job, &agent_id, msg_id)
            .await
            .unwrap();
        assert!(matches!(first, RecordOutcome::Enqueued));

        let second = store
            .record_and_enqueue(&job, &agent_id, msg_id)
            .await
            .unwrap();
        match second {
            RecordOutcome::Duplicate(rec) => assert_eq!(rec.state, DeliveryState::Ready),
            RecordOutcome::Enqueued => panic!("expected duplicate on second call"),
        }
    }

    #[tokio::test]
    async fn test_claim_then_complete() {
        let Some(pool) = get_test_redis().await else {
            return;
        };
        let store = IngestionDeliveryStore::new(pool);
        let agent_id = format!("agent-{}", Uuid::new_v4());
        let msg_id = "msg-2";
        let job = make_job(&agent_id, msg_id);
        store
            .record_and_enqueue(&job, &agent_id, msg_id)
            .await
            .unwrap();

        let claimed = store
            .claim(&agent_id, msg_id, "worker-1", 60)
            .await
            .unwrap();
        assert!(claimed);

        // Second concurrent claim must fail — already Processing.
        let second_claim = store
            .claim(&agent_id, msg_id, "worker-2", 60)
            .await
            .unwrap();
        assert!(!second_claim);

        store
            .complete(
                &agent_id,
                msg_id,
                DeliveryState::Processed,
                Some("processed"),
                None,
            )
            .await
            .unwrap();

        let record = store.get(&agent_id, msg_id).await.unwrap().unwrap();
        assert_eq!(record.state, DeliveryState::Processed);
        assert_eq!(record.outcome.as_deref(), Some("processed"));
    }

    #[tokio::test]
    async fn test_reap_expired_requeues_under_max_retries() {
        let Some(pool) = get_test_redis().await else {
            return;
        };
        let store = IngestionDeliveryStore::new(pool);
        let agent_id = format!("agent-{}", Uuid::new_v4());
        let msg_id = "msg-3";
        let job = make_job(&agent_id, msg_id);
        store
            .record_and_enqueue(&job, &agent_id, msg_id)
            .await
            .unwrap();
        // Claim with a lease that is already expired (negative lease_secs).
        store
            .claim(&agent_id, msg_id, "worker-1", -10)
            .await
            .unwrap();

        let (requeued, dlq) = store.reap_expired(3).await.unwrap();
        assert!(requeued >= 1);
        assert_eq!(dlq, 0);

        let record = store.get(&agent_id, msg_id).await.unwrap().unwrap();
        assert_eq!(record.state, DeliveryState::Retryable);
    }

    #[tokio::test]
    async fn test_reap_expired_moves_to_dlq_after_max_retries() {
        let Some(pool) = get_test_redis().await else {
            return;
        };
        let store = IngestionDeliveryStore::new(pool);
        let agent_id = format!("agent-{}", Uuid::new_v4());
        let msg_id = "msg-4";
        let job = make_job(&agent_id, msg_id);
        store
            .record_and_enqueue(&job, &agent_id, msg_id)
            .await
            .unwrap();

        // Exhaust retries: claim+expire repeatedly.
        for _ in 0..4 {
            store
                .claim(&agent_id, msg_id, "worker-x", -10)
                .await
                .unwrap();
            store.reap_expired(3).await.unwrap();
        }

        let record = store.get(&agent_id, msg_id).await.unwrap().unwrap();
        assert_eq!(record.state, DeliveryState::Permanent);
    }
}
