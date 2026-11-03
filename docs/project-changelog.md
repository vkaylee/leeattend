## 2026-11-03

### Enhancements

#### User-Based Behalf Delegation
**Component:** Attendance, overtime, leave, and approvals

- Delegation assignees now use workspace users (`assignee_user_id`) and no longer require an employee record.
- Creation validates workspace membership in the system database; non-members are rejected.
- Admin UI selects workspace members and loads all member pages before displaying assignees.
- Existing behalf assignments are preserved in a tenant-local rollback table, then cleared by migration `20261103000002_behalf_assignments_user_cutover`.

**Verification:** Backend membership tests and frontend delegation tests cover member-only creation, user selection, target scopes, and pagination behavior.

---

# Project Changelog

Detailed record of significant changes, features, and fixes in LeeAttend.

---

## 2026-03-11

### Enhancement

#### Adaptive Polling Simplification (agent-main)
**Component:** Edge Agent - Orchestrator & Adaptive Polling
**Status:** Complete

**Problem:**
- Adaptive polling used two overlapping mechanisms: a skip counter (cooldown after new data) and interval escalation (when no new data).
- After fetching new data, the logged interval showed `5s` but actual wait was ~300s via 60 skip cycles, creating confusing logs.
- Proactive disconnect after cooldown caused false "connection lost" warnings and inflated reconnect frequency counters.

**Solution:**
- Removed skip counter mechanism entirely (`skip_remaining`, `should_skip()`, `with_skip_polls()`).
- Implemented direct interval approach with `with_cooldown()` and `needs_disconnect()`.
- When new data found: interval = `SUCCESS_COOLDOWN_SECS` (300s), disconnect device, sleep, reconnect.
- When no new data: interval doubles (5→10→20→40→45s cap), stay connected within UDP session timeout.
- Added `proactive_cooldown` flag to distinguish planned vs error disconnects in outer reconnect loop.

**Files Changed:**
- `agent-main/src/adaptive_polling.rs` — Removed skip fields/methods, added `with_cooldown()`, `needs_disconnect()`
- `agent-main/src/config.rs` — Renamed `success_skip_polls` → `success_cooldown_secs`, env var `SUCCESS_COOLDOWN_SECS`
- `agent-main/src/orchestrator.rs` — Removed `should_skip()` branch, added proactive disconnect + cooldown flag
- `.env.example` — Updated env var reference
- `docker-compose.dev.yml` — Updated env var reference

**Breaking Changes:**
- Env var `SUCCESS_SKIP_POLLS` renamed to `SUCCESS_COOLDOWN_SECS` (default: 300)

**Tests:**
- All adaptive_polling unit tests updated and passing (100% line coverage)
- All config tests updated and passing (100% line coverage)
- Full smoke test suite: agent-main + apiserver (clippy + unit) all pass

---

## 2026-02-14

### Features

#### Dark Mode Phase 4: Feature Pages Completed (webdashboard)
**Component:** Web Dashboard - Feature Pages & Components
**Status:** Complete

**Implementation:**
- Added `dark:` variants to 91 files (30 page files, 61 component files)
- Applied 2178+ dark: Tailwind CSS v4 variants across pages and components
- Fixed auth page backgrounds: replaced hardcoded `bg-[#f8fafc]` with semantic `bg-slate-50 dark:bg-slate-950`
- Fixed gradient dark variants: primary/indigo gradients with proper opacity for dark mode
- Fixed shadow overrides: added `dark:shadow-black/40` for premium shadows in dark backgrounds
- Covered all pages: LoginPage, RegisterPage, DashboardPage, Reports, Devices, Settings, etc.
- Covered all components: Dialogs, Tables, Cards, Forms, Lists, Charts, Status badges

**Pages Modified (30):**
- LoginPage, RegisterPage, DashboardPage, ProfilePage, SettingsPage
- ReportsPage, AttendancePage, AnalyticsPage, PayrollPage
- DevicesPage, ConnectivityPage, ConfigurationPage
- ApproversPage, EmployeesPage, WorkspacesPage, etc.

**Components Modified (61):**
- Dialogs: BulkUploadOtDialog, EnterPaperOtDialog, AttendanceImportDialog, AgentEditModal, etc.
- Tables: DailyReportTable, MonthlyReportTable, TemporalConfigTable, etc.
- Cards: AgentAccordionCard, DeviceListItem, EmployeeCard, etc.
- Forms: EmployeeSearch, MonthYearPicker, PaginationControls, ReportFilters, etc.
- Layout guards: AuthGuard, WorkspaceGuard
- Other: HierarchyLegend, DebugToolbar, LanguageSwitcher adjustments

**Files Modified:** 91 TSX files (pages + components)

**Tests:**
- All 465 existing unit tests pass
- All pages render correctly in dark mode
- No contrast issues (WCAG AA compliance)
- Smooth transitions between light/dark modes
- All dialogs, modals, and overlays properly themed

**Validation:**
- Auth pages: Dark backgrounds, shadow overrides, readable text contrast
- Feature pages: Consistent dark styling across all dashboard views
- Tables & lists: Alternating rows, borders, hover states properly themed
- Form inputs: Dark borders, focus rings, placeholders styled for dark
- Buttons & badges: Status colors preserved with dark variants
- Charts & visualizations: Proper contrast in dark mode

---

#### Dark Mode Phase 3: UI Primitives Completed (webdashboard)
**Component:** Web Dashboard - Overlay Components
**Status:** Complete

**Implementation:**
- Added `dark:` variants to Modal, Toast, Combobox, Sheet, dropdown-menu, Tabs, LanguageSwitcher
- Modal: Content panel bg, header/footer borders, close button dark variants
- Toast: Container and variant colors (success/error/info) with dark backgrounds
- Combobox: Dropdown panel, search input, option rows, hover/select states
- Sheet: Slide-in panel bg, overlay backdrop
- dropdown-menu: Menu panel, item hover, separator dark variants
- Tabs: Tab bar bg, active/inactive tab states with dark colors
- LanguageSwitcher: Dropdown panel and selected state dark variants

**Files Modified:**
- `webdashboard/src/components/ui/Modal.tsx`
- `webdashboard/src/components/ui/Toast.tsx`
- `webdashboard/src/components/ui/Combobox.tsx`
- `webdashboard/src/components/ui/Sheet.tsx`
- `webdashboard/src/components/ui/dropdown-menu.tsx`
- `webdashboard/src/components/ui/Tabs.tsx`
- `webdashboard/src/components/ui/LanguageSwitcher.tsx`

**Tests:**
- All 465 existing unit tests pass
- All overlay components render correctly in dark mode
- No contrast issues or visual artifacts

**Validation:**
- Modal dialogs readable with dark backgrounds
- Toast notifications styled appropriately per variant (success/error/info)
- Combobox dropdown usable with dark search input and options
- Focus rings visible on all dark backgrounds
- Consistent styling across all primitive types

---

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
