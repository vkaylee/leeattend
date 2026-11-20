# Payroll impact input fingerprint evidence

## Rule IDs

- `API-01`: additive, explicit contract metadata.
- `TEST-01`: malformed, boundary, and policy-only provenance cases.
- `DB-02`: fingerprint computed from persisted snapshot input; no extra query or state.

## Contract

Impact comparison now exposes SHA-256 `input_fingerprint` for each computed result. Fingerprint canonicalizes JSON input and removes only top-level `policy` provenance, matching existing comparison semantics. Business input changes produce different fingerprints; policy-version-only changes preserve fingerprint and are represented by stale policy metadata.

Malformed input returns existing malformed/uncomputable behavior. No payroll state, activation state, or persisted fingerprint column is added.
