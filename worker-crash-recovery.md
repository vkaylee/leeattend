# Worker Crash Recovery Phase

- REL-01: Jobs must be resumable/idempotent; retries bounded and classified.
- OBS-01: Worker recovery state must be observable without PII or secrets.
- TEST-01: Cover crash between claim and completion, lease expiry, retry exhaustion, poison work, and replay.

Test impact:
- Success: claimed job completes and durable state updates once.
- Crash: abandoned lease becomes claimable after expiry; retry count increments.
- Duplicate: replay does not duplicate publication/evidence.
- Poison: bounded attempts move job to terminal dead-letter state without blocking healthy jobs.
- Concurrency: one lease owner processes a job; competing worker cannot claim active lease.
- Operations: worker heartbeat/live wiring remains visible to health/monitor endpoints.
