# Implementation Plan: Deprecate Legacy Time Tracking Page

**Status:** Draft
**Date:** 2026-01-21
**Objective:** Remove the legacy `TimeTrackingPage` (`/time-tracking`) as it has been superseded by the new Employee Self-Service (ESS) portal (`/ess/clock`) and Admin Attendance Logs (`/workspace/attendance/logs`).

## 📋 Approach
**Hard Removal:**
We will perform a complete removal of the page and its routing configuration. This is preferred over soft deprecation because the page encourages an "admin-as-kiosk" pattern that is no longer the intended workflow for the system.

## 🛠 Steps

### 1. Remove Page Component (5 min)
- **Delete:** `webdashboard/src/pages/TimeTrackingPage.tsx`
- **Verify:** Ensure no other files import this component (already checked via grep, only `App.tsx` found).

### 2. Update Routing & Navigation (5 min)
- **Modify:** `webdashboard/src/App.tsx`
  - Remove the import `import TimeTrackingPage from "./pages/TimeTrackingPage";`
  - Remove the route `<Route path="/time-tracking" element={<TimeTrackingPage />} />`
- **Modify:** `webdashboard/src/components/layout/Sidebar.tsx`
  - Remove the navigation item `{ to: "/time-tracking", icon: Clock, label: t("common.time_tracking") }`.

### 3. Cleanup Translations (5 min)
- **Modify:** `webdashboard/src/i18n/locales/en.json`, `vi.json`, `zh.json`.
  - Remove the top-level `time_tracking` object (all keys within it).
  - Remove `common.time_tracking` key if it's no longer used.

### 4. Verify Build (5 min)
- Run `npm run build` in `webdashboard/` to ensure no dangling references or type errors remain.

## ⏱ Timeline
| Phase | Duration |
|-------|----------|
| Removal | 5 min |
| Routing Update | 5 min |
| Cleanup | 5 min |
| Verification | 5 min |
| **Total** | **20 min** |

## 🔄 Rollback Plan
- `git checkout webdashboard/src/pages/TimeTrackingPage.tsx`
- Revert changes to `App.tsx`, `Sidebar.tsx`, and locale files.

## ✅ Verification Checklist
- [ ] Application compiles successfully.
- [ ] Sidebar no longer shows "Time Tracking".
- [ ] Accessing `/time-tracking` redirects to 404 (or Dashboard if caught by catch-all).
- [ ] No regression in other pages (ESS Clock, Attendance Logs).
