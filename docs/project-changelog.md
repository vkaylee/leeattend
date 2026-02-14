# Project Changelog

Detailed record of significant changes, features, and fixes in LeeAttend.

---

## 2026-02-14

### Features

#### Dark Mode Phase 2: Core Layout Completed (webdashboard)
**Component:** Web Dashboard - Layout Shell
**Status:** Complete

**Implementation:**
- Added `dark:` variants to MainLayout, Navbar, and Sidebar components
- MainLayout: Replaced hardcoded `bg-[#fcfcfd]` with `bg-background` semantic token; added dark mobile overlay
- Navbar: 50+ dark variants covering header, buttons, breadcrumbs, user area, notification bell
- Sidebar: Dark variants for container, nav links, active states, sections, logout button
- Glassmorphism effects (backdrop-blur) properly styled for dark backgrounds
- Mobile sidebar overlay: dark:bg-slate-950/60

**Files Modified:**
- `webdashboard/src/components/layout/MainLayout.tsx`
- `webdashboard/src/components/layout/Navbar.tsx`
- `webdashboard/src/components/layout/Sidebar.tsx`

**Tests:**
- All 465 existing unit tests pass
- Visual verification: toggle dark mode on all layout components
- No contrast issues (WCAG AA compliance)

**Validation:**
- All three core layout components render correctly in dark mode
- No layout shifts or visual artifacts
- Hover/focus/active states have proper dark variants

---

#### Dark Mode Phase 1: Foundation Completed (webdashboard)
**Component:** Web Dashboard - Theme Infrastructure
**Status:** Complete

**Implementation:**
- CSS dark variant registration via `@variant dark (&:where(.dark, .dark *))`
- `.dark` class CSS variable overrides (background, foreground, card, sidebar tokens)
- Semantic token registration in `@theme` for Tailwind integration
- FOUC prevention inline script in `index.html` with system preference detection
- `use-dark-mode.ts` hook: localStorage persistence, matchMedia sync, cross-tab sync
- DarkModeContext split into 3 files: provider, definition, consumer hook
- DarkModeProvider wired into App.tsx

**Files Created:**
- `webdashboard/src/hooks/use-dark-mode.ts` (213 lines)
- `webdashboard/src/context/DarkModeContext.tsx` (split into 3 files for ESLint compliance)

**Files Modified:**
- `webdashboard/src/index.css` (dark variant, CSS vars, semantic tokens)
- `webdashboard/index.html` (FOUC script, transition class)
- `webdashboard/src/App.tsx` (DarkModeProvider wrapper)

**Tests:**
- All 465 existing unit tests pass
- 2 E2E baseline tests validated
- ESLint: Clean, no issues

**Validation:**
- System preference detection verified
- Cross-tab sync confirmed
- Private browsing safety tested
- FOUC elimination validated on hard refresh

---

## 2026-02-13

### Bug Fixes

#### WebSocket ACK Starvation Fix (agent-main)
**Severity:** Critical
**Component:** Edge Agent - WebSocket Sync
**Impact:** Prevented logs from syncing when outgoing queue was busy

**Problem:**
- WS Connection Manager used single `tokio::select!` loop for both reading ACKs and writing outgoing messages
- When sync loop queued many outgoing messages, select loop starved read branch
- ACKs never received → logs stuck as unsynced indefinitely
- Affected high-volume installations with 500+ logs per sync cycle

**Solution:**
- Split into independent read/write tokio tasks
- Read task: Dedicated for processing incoming ACKs and server messages
- Write task: Dedicated for sending attendance logs and heartbeats
- Graceful shutdown via `oneshot` channel preserves `mpsc::Receiver` across reconnections
- Both tasks run concurrently, eliminating starvation

**Files Changed:**
- `agent-main/src/central_client/connection.rs` (lines 238-299)
  - Replaced single select loop with split read/write tasks
  - Added shutdown signal for graceful rx preservation
  - Write task returns rx on shutdown for reconnect reuse

**Tests Added:**
- `agent-main/tests/ws_split_readwrite_test.rs` (4 tests)
  - `test_concurrent_ack_processing_during_bulk_send` - Verifies ACKs processed during bulk send
  - `test_reconnect_after_server_close` - Validates rx preservation across reconnect
  - `test_connection_failure_preserves_receiver` - Ensures rx not lost on connection failure
  - `test_no_ack_starvation_under_load` - Load test with 50 logs and 2ms server delay

**Verification:**
- All 4 new tests pass consistently
- Total agent test suite: 182 tests (was 178)
- No regressions in existing WebSocket connection tests

**Related Documentation:**
- Updated `docs/system-architecture.md` section 2.1 with split read/write architecture

---

## Template for Future Entries

### [Feature|Bug Fix|Enhancement|Security]

#### [Short Title]
**Severity:** [Critical|High|Medium|Low]
**Component:** [Component Name]
**Impact:** [User/System Impact Description]

**Problem:** (for bug fixes)
- Detailed description

**Solution:**
- Implementation approach

**Files Changed:**
- File paths with brief description

**Tests Added:** (if applicable)
- Test file and descriptions

**Breaking Changes:** (if any)
- Migration steps

**Related Issues:** (if tracked)
- Issue numbers or links

---
