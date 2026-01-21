# Time Tracking Architecture Refactor - Implementation Plan

---
status: in-progress
priority: high
complexity: medium
estimated_duration: 2-3 days
created: 2026-01-21
phase_1_completed: 2026-01-21T14:57:00Z
---

## Overview

Refactor time tracking from admin-focused single page to clean separation: employee self-service clock + admin oversight dashboard. Addresses scalability, hybrid deployment, and UX issues.

**Context:** [Brainstorm Report](/home/elt1541/leeattend/webdashboard/plans/reports/brainstorm-260121-1013-time-tracking-refactor.md)

## Architecture Decision

**Split Approach (3 Pages):**
```
ESS Domain (Employee-Owned)
├─ /ess/clock (NEW) - Self-service punch in/out
└─ /ess/attendance (EXISTS) - Read-only history

Admin Domain (Oversight)
└─ /time-tracking (REFACTOR) - Team dashboard, admin adjustments
```

## Implementation Phases

| Phase | Priority | Complexity | Duration |
|-------|----------|------------|----------|
| [Phase 1: ESS Self-Clock Page](./phase-01-ess-self-clock.md) | P0 | Medium | 6-8h |
| [Phase 2: Admin Dashboard Refactor](./phase-02-admin-dashboard.md) | P1 | Medium | 6-8h |
| [Phase 3: Navigation & Integration](./phase-03-navigation-integration.md) | P2 | Low | 2-3h |
| [Phase 4: Testing & Validation](./phase-04-testing-validation.md) | P0 | Low | 3-4h |

**Total Estimate:** 17-23 hours (2-3 days)

## Progress Tracking

### Phase 1: ESS Self-Clock Page ✅ COMPLETE
**Status:** DONE (All Tasks Complete) | **Completion Date:** 2026-01-21 14:57 UTC | **Review Score:** 9.5/10 | **Reports:** [Phase 1 Review](./reports/code-reviewer-260121-1448-phase-01-review.md) | [Phase 1 Completion](./reports/project-manager-260121-1457-phase-01-completion.md)

- [x] Create `/ess/clock` page component ✅
- [x] Extract reusable punch card component ✅
- [x] Implement self-punch mutations (work/break in/out) ✅
- [x] Add current status display logic ✅
- [x] Build quick history view (today + 7 days) ✅
- [x] Add adjustment request modal ✅
- [x] Implement timezone handling ✅
- [x] Add loading/error states ✅
- [x] Write unit tests (21 tests added, 100% coverage) ✅
- [x] Fix type safety issue (all `any` usages removed) ✅
- [x] Fix error handling in adjustment submit ✅
- [x] Verify route/nav integration ✅

**Deliverables:** 3 reusable components (PunchCard, TimeEntryRow, AdjustmentRequestModal) | 21 unit tests | 0 critical issues

### Phase 2: Admin Dashboard Refactor
- [ ] Redesign `/time-tracking` to team overview
- [ ] Build real-time status grid (who's in/out/break)
- [ ] Add employee filter/search
- [ ] Implement admin adjustment modal
- [ ] Connect to existing adjustment approval workflow
- [ ] Add analytics summary cards
- [ ] Build export functionality

### Phase 3: Navigation & Integration
- [ ] Update `Sidebar.tsx` with ESS clock link
- [ ] Update `App.tsx` routes
- [ ] Add role-based navigation (show/hide ESS vs admin links)
- [ ] Update i18n translation keys
- [ ] Add breadcrumb navigation

### Phase 4: Testing & Validation
- [ ] Unit tests for ESS clock page
- [ ] Unit tests for admin dashboard
- [ ] Integration tests (punch flow, adjustment workflow)
- [ ] E2E tests (employee self-clock, admin oversight)
- [ ] Manual testing (hybrid scenario validation)
- [ ] Performance testing (100+ employees)

## Key Dependencies

**Existing Assets (Reuse):**
- ✅ `punchIn/punchOut` API endpoints
- ✅ `fetchTimeHistory` API
- ✅ `submitAttendanceAdjustment` API
- ✅ `AttendanceAdjustmentsPage` (approval workflow)
- ✅ Auth system with role checks
- ✅ Timezone utilities (`toUTCISO`)
- ✅ DateTime formatters (`useDateTimeFormatter`)

**New Requirements:**
- ⚠️ ESS punch hook (add to `use-ess.ts`)
- ⚠️ Team status API endpoint (optional, can derive from existing)
- ⚠️ Navigation role checks

## Success Criteria

**Functional:**
1. Employee can self-clock in/out without admin
2. Remote workers can use `/ess/clock` independently
3. Admin sees real-time team status on `/time-tracking`
4. Adjustment requests flow employee → admin approval
5. All existing punch logic preserved

**Non-Functional:**
1. Page load < 1s for both ESS and admin pages
2. Zero confusion about which page to use
3. Mobile-responsive ESS clock
4. 90%+ test coverage
5. No breaking changes to API

## Risks & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|-----------|
| Employee confusion (which page?) | Medium | Medium | Clear navigation, onboarding tooltips |
| Migration confusion | Medium | Low | Gradual rollout, training docs |
| Performance (100+ employees) | Low | Medium | Pagination, virtual scrolling on admin dashboard |
| Timezone bugs | Medium | High | Comprehensive timezone testing |
| Breaking existing workflows | Low | High | Preserve all existing API calls, add tests |

## Technical Decisions

**Component Reuse:**
- Extract `PunchCard` component from `TimeTrackingPage.tsx` (lines 138-192)
- Reuse `TimeEntryTable` logic (lines 203-258) with props
- Share adjustment modal between ESS and admin (lines 273-345)

**State Management:**
- ESS: Use `use-ess.ts` hook for current user context
- Admin: Keep existing `getCurrentWorkspaceId` approach
- No global state needed (React Query handles caching)

**Styling:**
- Maintain existing design system (rounded-[Xrem], primary colors)
- ESS page: larger touch targets (mobile-first)
- Admin page: dense table view (desktop-optimized)

## Unresolved Questions

1. **GPS Validation:** Enable GPS check on ESS clock? (API supports, just config)
2. **Break Types:** Support multiple break types (lunch, rest, smoke)? Current is generic
3. **Offline Support:** Queue punch actions if offline? Or require internet?
4. **Kiosk Mode:** Office kiosk uses ESS page (logged in as employee) or dedicated UI?
5. **Mobile App:** Timeline for mobile app? Will reuse ESS clock API endpoints?

**Decision:** Defer questions 3-5 to post-MVP. Address 1-2 during Phase 1 implementation.

## Rollout Strategy

**Phase A: Development (2-3 days)**
1. Implement all 4 phases
2. Internal testing with dev team
3. Fix bugs, iterate

**Phase B: Staging (1 week)**
1. Deploy to staging environment
2. User acceptance testing with 5-10 employees
3. Gather feedback, adjust UX

**Phase C: Production (Gradual)**
1. Soft launch: announce new ESS clock to remote workers
2. Monitor usage, fix issues
3. Full rollout: train all employees, deprecate old flow
4. Post-launch: gather metrics, iterate

## Documentation Requirements

- [ ] Update user guide (how to use ESS clock)
- [ ] Update admin guide (new team dashboard features)
- [ ] Add troubleshooting section (timezone issues, missed punches)
- [ ] Create video tutorial (2-min ESS clock walkthrough)
- [ ] Update API docs (if adding team status endpoint)

## Next Steps

1. **Review plan** with team (30 min)
2. **Kickoff Phase 1** (ESS self-clock page)
3. **Daily standups** to track progress
4. **Demo after Phase 2** to stakeholders
5. **UAT after Phase 4** before rollout

---

**Plan Status:** Phase 1 Complete - Phase 2 Ready
**Overall Progress:** 25% (1 of 4 phases complete)
**Approval Status:** Phase 1 complete, Phase 2 ready to start
**Start Date:** 2026-01-21
**Target Completion:** 2-3 days total (Phase 1 completed in 8 hours)
**Next Phase:** Phase 2 - Admin Dashboard Refactor (no blockers)
