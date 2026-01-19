# Implementation Plan: Pairing Traceability (Algorithm Audit Trail)

## Approach
We will add a `generation_metadata` column of type `JSONB` to the `time_entries` table. This allows us to store the algorithm name (e.g., "sequential"), version, and the IDs of the raw logs used to create the entry. 

**Why JSONB?**
- **Auditability:** Instantly see why an entry was created and from which logs.
- **Flexibility:** We can add extra debug info (like "calculated gap") later without database migrations.
- **Traceability:** Directly links derived `time_entries` back to their source `raw_attendance_logs`.

---

## Steps

### 1. Database Migration (5 min)
Create a new migration file: `apiserver/migrations/20260119000000_add_generation_metadata_to_time_entries.sql`
```sql
ALTER TABLE time_entries ADD COLUMN generation_metadata JSONB DEFAULT '{}'::jsonb;
```

### 2. Update Rust Models (10 min)
- Modify `TimeEntry` in `apiserver/src/models.rs` to include `generation_metadata: Option<serde_json::Value>`.
- Update `PairedSession` in `apiserver/src/services/pairing_algo.rs` to include a metadata field.

### 3. Update Pairing Strategies (15 min)
Modify the `PairingStrategy` trait and its implementations (`Sequential`, `DP`, etc.) to populate the metadata:
- Store the algorithm identifier.
- Capture the `log_ids` (requires passing log objects instead of just minutes to the algo, or mapping them back).

### 4. Update Pairing Service (10 min)
Modify `PairingService::reprocess_daily_attendance` in `apiserver/src/services/pairing_service.rs`:
- Update the `INSERT` query to save the `generation_metadata`.
- Map the resulting `PairedSession` metadata into the database call.

### 5. Testing & Verification (15 min)
- Update existing integration tests in `apiserver/tests/raw_attendance_pairing_tests.rs` to assert that the `generation_metadata` is correctly populated.
- Manually check the database via `psql` to verify the JSON structure.

---

## Timeline
| Phase | Duration |
|-------|----------|
| Migration | 5 min |
| Model Updates | 10 min |
| Logic Updates | 25 min |
| Testing | 15 min |
| **Total** | **~1 hour** |

---

## Rollback Plan
1. Create a "down" migration to drop the column:
   ```sql
   ALTER TABLE time_entries DROP COLUMN generation_metadata;
   ```
2. Revert the Rust model changes.

---

## Security Checklist
- [x] **Audit Trail:** JSONB provides a clear history of data derivation.
- [ ] **Auth Checks:** (Already handled by existing RBAC in handlers).
- [x] **Data Integrity:** Default to empty object `{}` to avoid null pointer issues in queries.
