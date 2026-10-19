# P4-11 OpenAPI Validation Evidence

## Contract

- Rust OpenAPI source remains SSOT: `apiserver/src/openapi.rs:1-1223`.
- Frontend generated contract baseline: `webdashboard/src/types/api-generated.ts`.
- Frozen counts and known mismatch: `docs/migration/rust-go/openapi-baseline.md`.
- Go validation entrypoint: `apiserver/internal/api.ValidateOpenAPIArtifact`.
- CLI validation entrypoint: `scripts/validate-openapi-contract.py ARTIFACT EXPECTED_SHA256`.

Both entrypoints fail closed when the artifact is absent or its SHA-256 differs. They do not generate a schema or silently accept drift.

## Verification

- `go test -race ./internal/api`: passed in `logs/run_p4_unit_final.log`.
- Missing artifact test: `TestValidateOpenAPIMissingArtifactFailsClosed` passed.
- Matching digest test: `TestValidateOpenAPIMatchingArtifact` passed.
- Current live Go generated OpenAPI artifact: **not available**. P4-11 therefore verifies fail-closed behavior, not schema parity.

## Limitation

A generated Go artifact and oapi-codegen configuration are required before P4-11 can claim empty schema diff. Creating a substitute schema would violate the Rust OpenAPI SSOT.
