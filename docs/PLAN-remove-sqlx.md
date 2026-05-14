# Plan: Complete Removal of SQLx

## Goal
Completely remove the `sqlx` dependency from the codebase, transitioning all database access and infrastructure strictly to Diesel ORM.

## Tasks
- [ ] Task 1: Audit and delete legacy SQLx repositories → Verify: Files like `user_repository.rs`, `time_entry_repository.rs`, etc. are removed, and `apiserver/src/repositories/mod.rs` is updated.
- [ ] Task 2: Complete migration of mixed repositories → Verify: `workspace_stats_repository.rs` and `diesel_payroll_rule_repository.rs` no longer require `PgPool` in their constructors or methods.
- [ ] Task 3: Remove `sqlx` derivations from Models → Verify: Delete `#[derive(sqlx::FromRow)]` and replace `sqlx::types::Json` with `serde_json::Value` or `diesel::sql_types::Jsonb` across all files in `apiserver/src/models/` and `apiserver/src/domain/`.
- [ ] Task 4: Refactor Database Infrastructure and Tests → Verify: `db_pool.rs`, `TestDatabase`, and `TestFactory` only manage Diesel connections; no `PgPool` initialization exists.
- [ ] Task 5: Clean up App State and Builders → Verify: `app_builder.rs` and service constructors no longer accept `sqlx::PgPool` arguments.
- [ ] Task 6: Purge `sqlx` dependencies and caches → Verify: Remove `sqlx` from `Cargo.toml`, delete `.sqlx/` cache directory, and remove `cargo sqlx prepare` from `./test.sh` and CI scripts.

## Done When
- [ ] No occurrences of `sqlx::` exist in the entire `apiserver/src/` or `apiserver/tests/` directories.
- [ ] `cargo check --tests` compiles successfully without the `sqlx` crate.
- [ ] The integration test suite (`./test.sh api`) passes fully and no longer executes the `sqlx prepare` step.
