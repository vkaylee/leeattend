# Tenant transaction chokepoint refactor

Status: implemented and verified on the tenant-migration branch.

## Rule compliance

- `DB-ISOLATION-01` — `.agent/rules/database-rules.md §1`: tenant-scoped queries remain workspace-scoped.
- `DB-TX-01` — `.agent/rules/database-rules.md §3`: multi-mutation operations use an atomic transaction.
- `ERR-DIESEL-01` — `.agent/rules/coding-standards.md §3`: repository database errors propagate as `AppError` without repetitive lossy mappings.
- `EXEC-HERMETIC-01` — `.agent/rules/execution-wrappers.md §2`: tests/builds use project wrappers with redirected logs.
- `TEST-FULL-01` — `.agent/rules/testing-standards.md §2`: each migration batch runs the full API suite.
- `ENCAPSULATION-01` — `.agent/rules/coding-standards.md §2`: `DieselPools` tenant providers are private implementation details.

## Design gate

Diesel Async 0.8's transaction API requires a higher-ranked callback returning a `ScopedBoxFuture`:

```rust
pub async fn with_tenant_tx<'a, T, F>(
    &self,
    token: &TenantAuthToken,
    f: F,
) -> Result<T, AppError>
where
    T: Send + 'a,
    F: for<'conn> FnOnce(
            &'conn mut AsyncPgConnection,
        ) -> diesel_async::scoped_futures::ScopedBoxFuture<
            'a,
            'conn,
            Result<T, AppError>,
        > + Send
        + 'a,
{
    // acquire tenant transaction pool connection, then invoke
    // AsyncConnection::transaction; implemented in DieselPoolsTenantTxExt.
}
```

The migration must preserve existing transaction-aware `_tx` call chains. In particular,
`replace_time_entries_tx` and `batch_upsert_calculated_summaries_tx` receive a connection
from callers that already own the outer transaction; they must not open an additional
transaction in their current form.

## Planned implementation order after approval

1. Add and compile-test the wrapper plus Diesel error conversion.
2. Migrate the four highest-density repositories one file at a time: leave policy,
   attendance daily summary, time entry, and employment.
3. Run the required full API suite and commit after each file/batch.
4. Migrate remaining repository bypasses until `tenants_tx`/`tenants_sess` can become private.
5. Add atomic boundaries to the explicitly identified multi-mutation methods, excluding
   transaction-aware helpers whose caller-owned transaction contract is confirmed.
6. Remove repetitive `db: {e}` mappings while preserving intentional domain-specific
   `NotFound`/conflict matches.
7. Compile and run the full suite sequentially; inspect each phase's result rather than
   relying on the summary line.
