# Task list

## Completed workspace cleanup

- [x] Remove active workspace/tenant wording, BYOD schema field, stale worker docs, obsolete coverage/security references.
- [x] Preserve outbox, Redis, schedule/report workers, auth, and business scopes.
- [x] Remove unused generic in-memory worker after call-site proof.
- [x] Run security audit, targeted tests, and full regression before contract work.

## Go-owned OpenAPI contract

- [x] C0 Freeze 426 Go route-method inventory (406 registerRoute + 2 direct health plus duplicate declarations) and classify active flat routes.
- [x] C1 Add `apiserver/openapi.yaml` as Go-owned source; cover current route inventory and reject SaaS workspace/tenant/BYOD/platform-role surfaces.
- [x] C2 Add deterministic generation inputs: `scripts/generate_openapi.py`, `scripts/validate_openapi_contract.py`, package generator path, pinned container generation.
- [x] C3 Finish consumer migration after generated schema review; generated output has no stale SaaS terms and frontend `tsc --noEmit` passes.
- [x] C4 Replace stale CI generated-code check with source parity + generated freshness gates.
- [ ] C5 Update contract docs and run focused + full verification.

## Checkpoints

- [x] Previous workspace cleanup checkpoints passed.
- [ ] C0 reviewed route inventory.
- [ ] C1 spec parses and route parity passes.
- [ ] C2 deterministic generation passes twice.
- [ ] C3 frontend typecheck/unit passes.
- [ ] C4 CI drift and forbidden-surface checks pass.
- [ ] C5 full `./leedevkit test all` passes.
