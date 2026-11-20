# Payroll impact input fingerprint evidence

## Rule IDs

- `API-01`: additive, explicit contract metadata.
- `TEST-01`: malformed, boundary, and policy-only provenance cases.
- `DB-02`: fingerprint computed from persisted snapshot input; no extra query or state.

## Contract

Impact comparison now exposes SHA-256 `input_fingerprint` for each computed result. Fingerprint canonicalizes JSON input and removes only top-level `policy` provenance, matching existing comparison semantics. Business input changes produce different fingerprints; policy-version-only changes preserve fingerprint and are represented by stale policy metadata.

Malformed input returns existing malformed/uncomputable behavior. No payroll state, activation state, or persisted fingerprint column is added.

## Step 6 simulation safeguards

- Aggregate impact requires `payroll:sensitive:read`; unauthorized callers are rejected before repository work.
- Batch input is bounded to 100 employment IDs, rejects duplicates, invalid periods, nil/equal run IDs, and oversized JSON bodies.
- Missing, formula-linked, mismatched, malformed, or incomplete runs remain explicit `uncomputable` items; they never become zero/unaffected.
- Comparison is read-only and transaction-free: repeated requests recompute from immutable calculation-run snapshots and cannot create payslips, runs, or payroll periods. No retry queue or idempotency key is needed for this read-only operation.
- Simulation itself executes inside a transaction and existing integration coverage verifies rollback of payslips, calculation runs, and payroll periods. Resource/time-limit expansion remains deferred until measured demand requires queueing or a separate batch simulation contract.
