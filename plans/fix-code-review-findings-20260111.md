# Implementation Plan: Fix Code Review Findings (2026-01-11)

## Approach
- **Consolidate `agent-zk-worker`**: Eliminate duplication between `main.py` and `zk_worker.py`.
- **Remove Dead Code**: Delete unused legacy files (`models.py`, `device_manager.py`) and their tests to simplify the architecture.
- **Harden Backend**: Replace `unwrap()` with safe error handling in payroll logic.
- **Type Safety**: Fix `any` usage in frontend mutations.

## Steps

### 1. Python Worker Refactoring (agent-zk-worker)
- **Files to modify**: `agent-zk-worker/main.py`
- **Files to delete**: 
    - `agent-zk-worker/models.py`
    - `agent-zk-worker/device_manager.py`
    - `agent-zk-worker/tests/test_database.py`
    - `agent-zk-worker/tests/test_memory_safety.py`
    - `agent-zk-worker/tests/test_safety.py`

### 2. Backend Error Handling (apiserver)
- **File to modify**: `apiserver/src/services/payroll_service.rs`
- **Change**: Replace `.unwrap()` on `from_ymd_opt` and `pred_opt` with `ok_or`.

### 3. Frontend Type Safety (webdashboard)
- **File to modify**: `webdashboard/src/pages/WorkspaceSettingsPage.tsx`
- **Change**: Replace `any` in `useMutation` with `Partial<Workspace>`.

## Timeline
| Phase | Duration |
|-------|----------|
| Cleanup & Refactor | 15 min |
| Backend Hardening | 10 min |
| Frontend Types | 5 min |
| Verification | 10 min |
| **Total** | **40 min** |

## Rollback Plan
- Use `git checkout .` to revert to the current stable state.

## Security Checklist
- [x] Input validation (Date parameters)
- [x] Error handling (No panics)
- [x] Code integrity (Removing unused/broken files)
