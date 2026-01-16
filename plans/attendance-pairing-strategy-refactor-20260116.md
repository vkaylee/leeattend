# Implementation Plan: Attendance Pairing Strategy Refactor

### ## Approach
- **Why this solution:** Currently, the pairing logic is dispatched via `match` statements in the service layer. As we add more algorithms, this becomes hard to maintain. Refactoring into a **Strategy Pattern** allows us to encapsulate each algorithm's logic and test them against a unified "Golden Dataset" of scenarios.
- **Alternatives considered:** Keeping the `match` statement (simple, but doesn't scale for complex testing) or using a trait-based factory (chosen solution).

### ## Steps

1. **Define Strategy Trait** (10 min)
   - Modify `apiserver/src/services/pairing_algo.rs`.
   - Define `pub trait PairingStrategy`:
     ```rust
     pub trait PairingStrategy {
         fn pair(&self, logs: &[i32], config: Option<&PairingShiftConfig>) -> Vec<PairedSession>;
     }
     ```

2. **Refactor Existing Logic into Structs** (30 min)
   - Create `struct DpPatternStrategy` and `struct AnchorFirstStrategy`.
   - Move existing code from `pair_logs_min_cost` and `pair_anchor_first` into their respective trait implementations.

3. **Update Dispatcher** (10 min)
   - Add a factory function in `pairing_algo.rs`:
     ```rust
     pub fn get_strategy(algorithm: &str) -> Box<dyn PairingStrategy> {
         match algorithm {
             "anchor_first" => Box::new(AnchorFirstStrategy),
             _ => Box::new(DpPatternStrategy),
         }
     }
     ```

4. **Integration** (15 min)
   - Update `apiserver/src/services/pairing_service.rs` to use the factory.

5. **Implement Shared Test Suite** (30 min)
   - Create a test helper that defines standard scenarios.
   - Run both strategies against these scenarios.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Trait Definition | 10 min |
| Refactoring | 30 min |
| Integration | 15 min |
| Shared Testing | 30 min |
| **Total** | **1 hour 25 min** |

### ## Rollback Plan
- Revert changes using `kit_restore_checkpoint`.

### ## Security Checklist
- [x] **Correctness:** Ensure `PairingShiftConfig` is passed correctly.
- [x] **Stability:** Default to DP algorithm for unknown inputs.
