# Implementation Plan: FSM (State-Based) Pairing Strategy

### ## Approach
- **Goal:** Support hardware/apps that provide explicit punch types (Check-In, Check-Out, Break).
- **Mechanism:** Finite State Machine that processes logs sequentially, respecting the `punch_type` metadata.
- **Architecture Refactor:** Change `PairingStrategy` to accept a list of `LogEvent` structs (containing minutes + type) instead of just `i32` minutes.

### ## Steps

1. **Refactor Strategy Input** (backend)
   - Define `LogEvent` struct in `pairing_algo.rs`.
   - Update `PairingStrategy` trait definition.
   - Update `DpPatternStrategy`, `AnchorFirstStrategy`, and `SequentialStrategy` to use `LogEvent`.

2. **Implement FsmStrategy** (backend)
   - Add logic to handle explicit In/Out states.
   - Handle missing metadata by toggling state.

3. **Update PairingService** (backend)
   - Fetch `punch` (status) from `raw_attendance_logs`.
   - Pass `Vec<LogEvent>` to the strategy.

4. **Frontend Selection** (React UI)
   - Add "State-based (FSM)" option to Workspace Settings.

5. **Localization** (en, vi, zh)
   - Add translations for FSM strategy.

6. **Testing** (Integration tests)
   - Add tests for explicit state sequences and error recovery.

### ## Success Criteria
- Logs with `punch_type=0` always start a session.
- Logs with `punch_type=1` always end a session.
- Mixed logs (some with metadata, some without) are paired gracefully.
