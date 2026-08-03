# Dashboard reload loop fix

## Applicable rules

- `coding-standards.md`: strict TypeScript and resilient frontend state handling.
- `testing-standards.md`: add regression coverage and run the web test wrapper.
- `execution-wrappers.md`: use project wrappers with redirected logs for validation.

## Scope

- Keep chunk retry state across React mounts.
- Reset the retry budget only after the app has remained stable.
- Add regression tests for the reload-loop prevention behavior.
