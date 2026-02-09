# Implementation Plan: utoipa + openapi-typescript Integration

## Approach
- Use `utoipa` in the Rust `apiserver` to generate an OpenAPI 3.0 specification.
- Use `openapi-typescript` in `webdashboard` to generate type definitions from the spec.
- Automate the sync process via a script.

## Steps
1. **Backend Dependencies**: Add `utoipa` to `apiserver/Cargo.toml`.
2. **Schema Annotation**: Annotate Rust structs and handlers.
3. **Spec Generation**: Create a mechanism to export `openapi.json`.
4. **Frontend Integration**: Install `openapi-typescript` and generate types.
5. **Refactor**: Gradually replace manual types with generated ones.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend | 60 min |
| Frontend | 30 min |
| Verification| 30 min |

## Rollback
Revert file changes and continue using manual types.
