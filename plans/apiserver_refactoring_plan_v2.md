# Updated APIServer Refactoring Plan

During Phase 1 execution (Optimizing Axum State Management), we encountered Rust's **Orphan Rule (E0117)**. 
Rust prevents implementing a foreign trait (`axum::extract::FromRef`) on a foreign type (`std::sync::Arc<T>`).

Therefore, we cannot do `impl FromRef<Arc<AppState>> for Arc<EmployeeState>`.

## Proposed Alternative for Axum State
Instead of implementing `FromRef` for `Arc<EmployeeState>`, we can define a custom wrapper type local to our crate:
```rust
#[derive(Clone)]
pub struct AppRef(pub Arc<AppState>);

// Then implement FromRef for our custom wrapper targeting standard Arc types
impl FromRef<AppRef> for Arc<EmployeeState> {
    fn from_ref(state: &AppRef) -> Self {
        Arc::new(EmployeeState {
            service: state.0.employees.service.clone(),
            repo: state.0.employees.repo.clone(),
        })
    }
}
```
*Note: This still requires `Arc<EmployeeState>` to be cheap to clone (which it is, since it only contains `Arc`s).*
*Alternatively, we skip the Axum State Optimization (Phase 3 of the original plan) for now and focus strictly on splitting the God-files (Phase 1 of the original plan).*

## Adjusted Execution Plan
1. **(Immediate)**: Focus on Action Item 2 from the previous plan: Splitting the `employee_repository.rs` (45 KB) and `roster_repository.rs` (41 KB) into smaller trait abstractions. This provides the highest ROI for maintainability.
2. **(Deferred)**: Re-evaluate the Axum state extraction wrapper (`AppRef`) after the repository structures are stabilized.
