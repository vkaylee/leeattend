# Integration Constructor Migration

## Scope

Update integration tests and any additional test files surfaced by compilation to use the refactored `New*Params` constructor APIs, preserving test behavior and fixing directly related compile issues.

## Applicable Rules

- `CORE-5`: Follow PTY-safe command execution and log redirection.
- `CORE-9`: Run integration/full-suite verification; unit-only results are insufficient.
- `TEST-3`: Use the mandatory lint, unit, and full test workflow through `./leedevkit`.
- `CODE-2`: Read files before editing and keep Rust changes idiomatic and focused.

## Verification

1. Inspect constructor definitions and affected call sites.
2. Update imports and constructor calls.
3. Run `./scripts/_cargo.sh check --tests -p apiserver`.
4. Run `./leedevkit test api` and report every phase accurately.
