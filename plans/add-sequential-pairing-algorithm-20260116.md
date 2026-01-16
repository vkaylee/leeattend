# Implementation Plan: Sequential (Retail) Pairing Algorithm

### ## Approach
- **Goal:** Support environments with split shifts or multiple daily entries/exits.
- **Mechanism:** Chronological log pairing (Log 1+2, Log 3+4).
- **Architecture:** Extend `PairingStrategy` trait in the backend and update the algorithm selection UI.

### ## Steps

1. **Implement SequentialStrategy** (backend)
2. **Update Strategy Factory** (backend)
3. **Frontend Selection** (React UI)
4. **Localization** (en, vi, zh)
5. **Testing** (Integration tests)

### ## Success Criteria
- Employees with 4 logs have 2 logical sessions.
- Employees with 3 logs have 1 logical session and 1 "Missing Out" entry.
- UI allows switching to this algorithm and persists the setting.
