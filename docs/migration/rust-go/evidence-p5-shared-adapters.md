# Phase 5 Shared Adapters and Cross-Cutting Behavior Evidence

Status: Complete and verified against Go build gates.

## Tasks Covered
- **P5-01**: `internal/service/tx.go` - WithTx explicit commit, rollback on error, panic recovery, context cancellation.
- **P5-02**: `internal/tenant/scope.go` - WorkspaceScope non-nil UUID validation and static column predicate binding.
- **P5-03**: `internal/api/pagination.go` - ParsePagination defaults (page 1, limit 50, max 3000), SortOption allowlisting.
- **P5-04**: `internal/api/validation.go` - DecodeJSON with DisallowUnknownFields and single JSON token enforcement, ParseUUID, ParseDecimal, ParseDate, ParseTimezone.
- **P5-05**: `internal/audit/writer.go` - Asynchronous, fail-open, capacity-bounded audit writer with automatic PII redaction (password, secret, token, email, phone) and clean drain during Shutdown.
- **P5-06**: `internal/infra/external/adapters.go` - HTTPClient with timeout and 5xx error mapping, FileStore with path-traversal prevention, ClamAV INSTREAM protocol scanner, SMTPMailer.
- **P5-07**: `internal/queue/worker.go` - Background worker with exponential retry backoff, bounded queue channel, context cancellation, and graceful shutdown.
- **P5-08**: In-process pure domain packages with zero database, zero network, zero wall clock, and zero float arithmetic:
  - `internal/domain/pairing/pair.go` - Multi-segment, shift window, and anchor-first pairing algorithms.
  - `internal/domain/matcher/matcher.go` - Late arrival, early departure, absence detection, overnight shift window resolution.
  - `internal/domain/payroll/rates/rates.go` - Zero-safe daily/hourly rate formulas and proportional monthly overtime capping with scale 0 rounding.
  - `internal/domain/payroll/buckets/buckets.go` - Frame intersection break deduction, standard/early/late overtime minute buckets, integer rounding policies.
  - `internal/domain/payroll/incentives/evaluator.go` - Dynamic rules array and flat rule evaluation using arbitrary precision `shopspring/decimal`.
  - `internal/domain/payroll/rules/evaluator.go` - Percentage (with cap), fixed, and multi-tier statutory deduction brackets with zero suppression.
  - `internal/domain/payroll/payroll.go` - Payslip assembly and calendar month sub-segment partitioning.
  - `internal/domain/timezone/timezone.go` - Strict IANA timezone validation, lenient UTC fallback, local-to-UTC mapping, and DST-safe calendar day boundaries.
- **P5-09**: `internal/api/export.go` - CSV streaming with security headers (`X-Content-Type-Options: nosniff`, `Content-Disposition`) and row-limited CSV ingestion.

## Verification
- All Go quality gates (`./scripts/_go-gates.sh`):
  - `gofmt`: Clean across all packages.
  - `go vet`: Clean across all packages.
  - `staticcheck`: Zero warnings.
  - `govulncheck`: Zero vulnerabilities.
  - `go mod verify`: All modules verified.
  - `go test -race`: 100% PASS with race detector enabled across all packages.
