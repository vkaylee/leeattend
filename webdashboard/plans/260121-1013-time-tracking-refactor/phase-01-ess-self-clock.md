# Phase 1: ESS Self-Clock Page

**Priority:** P0
**Complexity:** Medium
**Duration:** 6-8 hours (COMPLETED IN 8 HOURS)
**Status:** ✅ COMPLETE
**Completion Date:** 2026-01-21 14:57 UTC
**Code Review:** [Phase 1 Review Report](./reports/code-reviewer-260121-1448-phase-01-review.md)
**Review Score:** 9.5/10 (Improved post-fixes)

## Overview

Create `/ess/clock` page for employee self-service punch in/out. Extract reusable logic from current `TimeTrackingPage.tsx`, remove admin employee selector, always use current user context.

## Objectives

1. Employee can clock in/out without admin intervention
2. Real-time status display (clocked in/out/break)
3. Quick history view (today + last 7 days)
4. Request adjustment button
5. Mobile-responsive design

## Technical Approach

### Component Structure

```
src/pages/ess/
├─ MyClockPage.tsx (NEW - main ESS clock page)
└─ MyAttendancePage.tsx (EXISTS - history)

src/components/time-tracking/ (NEW directory)
├─ PunchCard.tsx (EXTRACT from TimeTrackingPage)
├─ TimeEntryRow.tsx (EXTRACT from TimeTrackingPage)
├─ AdjustmentRequestModal.tsx (EXTRACT from TimeTrackingPage)
└─ StatusBadge.tsx (NEW - reusable status badge)
```

### Reuse Strategy

**From `TimeTrackingPage.tsx`** (lines to extract):
- **Punch Card UI** (lines 138-192): Clock in/out buttons, status display
- **History Table** (lines 195-260): Recent punch entries
- **Adjustment Modal** (lines 273-345): Request adjustment form
- **State Logic** (lines 68-95): Active entry detection, mutations

**Remove:**
- Employee selector (lines 121-133)
- Admin-specific features

**Adapt:**
- Always use current user (no `selectedEmployeeId` prop)
- Larger touch targets for mobile
- Simplified layout (single column on mobile)

## Implementation Steps

### Step 1: Create ESS Punch Hook (2h)

**File:** `src/hooks/use-ess.ts`

**Add:**
```typescript
// --- Clock Hook ---
export function useMyClock() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();

  // Fetch current user's time entries (today + 7 days)
  const { startDate, endDate } = useMemo(() => {
    const now = new Date();
    const end = now.toISOString();
    const start = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
    return { startDate, endDate };
  }, []);

  const entriesQuery = useQuery({
    queryKey: essKeys.clockEntries(),
    queryFn: () => fetchMyTimeEntries({ start_date: startDate, end_date: endDate }),
    refetchInterval: 30000, // Refresh every 30s
  });

  const activeEntry = entriesQuery.data?.find(e => !e.punch_out_time);
  const isOnBreak = activeEntry?.entry_type === "break";

  const punchInMutation = useMutation({
    mutationFn: (data: { entry_type: "work" | "break"; note?: string }) =>
      punchInMySelf(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: essKeys.clockEntries() });
      queryClient.invalidateQueries({ queryKey: essKeys.attendance() });
    },
  });

  const punchOutMutation = useMutation({
    mutationFn: (data?: { note?: string }) => punchOutMySelf(wsId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: essKeys.clockEntries() });
      queryClient.invalidateQueries({ queryKey: essKeys.attendance() });
    },
  });

  return {
    entries: entriesQuery.data || [],
    activeEntry,
    isOnBreak,
    isLoading: entriesQuery.isLoading,
    isError: entriesQuery.isError,
    error: entriesQuery.error,

    punchIn: punchInMutation.mutateAsync,
    punchOut: punchOutMutation.mutateAsync,
    isPunching: punchInMutation.isPending || punchOutMutation.isPending,
  };
}
```

**API Functions to Add** (`src/lib/api/ess.ts`):
```typescript
export async function fetchMyTimeEntries(params?: { start_date?: string; end_date?: string }) {
  const query = params ? `?${new URLSearchParams(params)}` : "";
  const response = await apiFetch(`${API_BASE_URL}/ess/time-entries${query}`);
  if (!response.ok) throw new Error("Failed to fetch time entries");
  return response.json();
}

export async function punchInMySelf(wsId: string, data: { entry_type: "work" | "break"; note?: string }) {
  const response = await apiFetch(`${API_BASE_URL}/ess/punch-in`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch in");
  return response.json();
}

export async function punchOutMySelf(wsId: string, data?: { note?: string }) {
  const response = await apiFetch(`${API_BASE_URL}/ess/punch-out`, {
    method: "POST",
    body: JSON.stringify(data || {}),
  });
  if (!response.ok) throw new Error("Failed to punch out");
  return response.json();
}
```

### Step 2: Extract Reusable Components (2h)

**2.1: PunchCard Component**

**File:** `src/components/time-tracking/PunchCard.tsx`

```typescript
interface PunchCardProps {
  activeEntry?: TimeEntry;
  isOnBreak: boolean;
  userName: string;
  onPunchIn: (type: "work" | "break") => Promise<void>;
  onPunchOut: () => Promise<void>;
  onRequestAdjustment: () => void;
  isPunching: boolean;
}

export function PunchCard({
  activeEntry,
  isOnBreak,
  userName,
  onPunchIn,
  onPunchOut,
  onRequestAdjustment,
  isPunching
}: PunchCardProps) {
  // Extract lines 138-192 from TimeTrackingPage.tsx
  // Adapt for props instead of state
  // Add mobile-optimized touch targets (min 48x48px)
}
```

**2.2: TimeEntryRow Component**

**File:** `src/components/time-tracking/TimeEntryRow.tsx`

```typescript
interface TimeEntryRowProps {
  entry: TimeEntry;
  onAdjust: (entry: TimeEntry) => void;
  showAdjustButton?: boolean; // Admin can adjust, employee can't
}

export function TimeEntryRow({ entry, onAdjust, showAdjustButton = false }: TimeEntryRowProps) {
  // Extract single row logic from lines 218-255
  // Mobile-responsive columns
}
```

**2.3: AdjustmentRequestModal Component**

**File:** `src/components/time-tracking/AdjustmentRequestModal.tsx`

```typescript
interface AdjustmentRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  timeEntry?: TimeEntry;
  onSubmit: (data: AdjustmentData) => Promise<void>;
  timezone: string;
}

export function AdjustmentRequestModal({ ... }: AdjustmentRequestModalProps) {
  // Extract lines 273-345 from TimeTrackingPage.tsx
  // Make modal reusable for both ESS and admin
}
```

### Step 3: Build MyClockPage (3h)

**File:** `src/pages/ess/MyClockPage.tsx`

```typescript
export default function MyClockPage() {
  const { t } = useTranslation();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const toast = useToast();
  const { employee } = useMyProfile();
  const { data: workspace } = useQuery({
    queryKey: ["workspace", getCurrentWorkspaceId()],
    queryFn: () => fetchWorkspace(getCurrentWorkspaceId()!),
  });

  const {
    entries,
    activeEntry,
    isOnBreak,
    isLoading,
    punchIn,
    punchOut,
    isPunching
  } = useMyClock();

  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);
  const [selectedEntry, setSelectedEntry] = useState<TimeEntry | undefined>();

  const handlePunchIn = async (type: "work" | "break") => {
    await punchIn({ entry_type: type });
    toast.success(t("ess.clock.punch_success"));
  };

  const handlePunchOut = async () => {
    await punchOut();
    toast.success(t("ess.clock.punch_out_success"));
  };

  const handleAdjustmentSubmit = async (data: AdjustmentData) => {
    await submitAttendanceAdjustment(getCurrentWorkspaceId()!, {
      employee_id: employee!.id,
      ...data
    });
    setIsAdjustmentModalOpen(false);
    toast.success(t("ess.clock.adjustment_requested"));
  };

  if (isLoading) {
    return <LoadingSkeleton />;
  }

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div>
          <h1 className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">
            {t("ess.clock.title")}
          </h1>
          <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60 mt-1.5">
            {t("ess.clock.subtitle")}
          </p>
        </div>
      </div>

      {/* Main Content: 2 Columns on Desktop, Stack on Mobile */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        {/* Left: Punch Card */}
        <div className="lg:col-span-5">
          <PunchCard
            activeEntry={activeEntry}
            isOnBreak={isOnBreak}
            userName={`${employee?.first_name} ${employee?.last_name}`}
            onPunchIn={handlePunchIn}
            onPunchOut={handlePunchOut}
            onRequestAdjustment={() => setIsAdjustmentModalOpen(true)}
            isPunching={isPunching}
          />
        </div>

        {/* Right: Quick History */}
        <div className="lg:col-span-7 bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm">
          <div className="px-8 py-6 border-b border-slate-50 bg-slate-50/30 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <History className="w-5 h-5 text-primary-500" />
              <h3 className="font-black text-slate-800 uppercase tracking-widest text-xs">
                {t("ess.clock.recent_history")}
              </h3>
            </div>
          </div>
          <div className="overflow-y-auto max-h-[600px]">
            <table className="w-full">
              <thead className="sticky top-0 bg-white/80 backdrop-blur-md border-b border-slate-100">
                <tr>
                  <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-left">
                    {t("common.date")}
                  </th>
                  <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-left">
                    {t("common.in")}
                  </th>
                  <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-left">
                    {t("common.out")}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {entries.length === 0 ? (
                  <tr>
                    <td colSpan={3} className="px-10 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
                      {t("ess.clock.no_entries")}
                    </td>
                  </tr>
                ) : (
                  entries.map(entry => (
                    <TimeEntryRow
                      key={entry.id}
                      entry={entry}
                      onAdjust={(e) => {
                        setSelectedEntry(e);
                        setIsAdjustmentModalOpen(true);
                      }}
                      showAdjustButton={false} // ESS users can't adjust directly
                    />
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Adjustment Request Modal */}
      <AdjustmentRequestModal
        isOpen={isAdjustmentModalOpen}
        onClose={() => setIsAdjustmentModalOpen(false)}
        timeEntry={selectedEntry}
        onSubmit={handleAdjustmentSubmit}
        timezone={workspace?.timezone || "UTC"}
      />
    </div>
  );
}
```

### Step 4: Add Route & Navigation (1h)

**4.1: Update Routes** (`src/App.tsx`):
```typescript
// Inside ESS routes section
<Route path="/ess/clock" element={<MyClockPage />} />
<Route path="/ess/attendance" element={<MyAttendancePage />} />
<Route path="/ess/profile" element={<MyProfilePage />} />
```

**4.2: Update Sidebar** (`src/components/layout/Sidebar.tsx`):
```typescript
// ESS Section
{ to: "/ess/clock", icon: Clock, label: t("ess.clock.title") },
{ to: "/ess/attendance", icon: CalendarDays, label: t("ess.attendance.title") },
{ to: "/ess/profile", icon: User, label: t("ess.profile.title") }
```

### Step 5: Add i18n Translations (30min)

**File:** `src/i18n/locales/en.json`

```json
{
  "ess": {
    "clock": {
      "title": "Time Clock",
      "subtitle": "Clock in and out for work and breaks",
      "punch_success": "Clocked in successfully",
      "punch_out_success": "Clocked out successfully",
      "adjustment_requested": "Adjustment request submitted",
      "no_entries": "No recent time entries",
      "recent_history": "Recent Activity (Last 7 Days)"
    }
  }
}
```

**Repeat for:** `vi.json`, `zh.json`

## Testing Requirements

### Unit Tests

**File:** `src/pages/ess/__tests__/MyClockPage.test.tsx`

```typescript
describe("MyClockPage", () => {
  it("renders punch card when user is clocked out", async () => {
    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/time clock/i)).toBeDefined();
    expect(screen.getByText(/off duty/i)).toBeDefined();
  });

  it("shows working status when user is clocked in", async () => {
    // Mock active entry
    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/working/i)).toBeDefined();
  });

  it("handles punch in action", async () => {
    const user = userEvent.setup();
    renderWithProviders(<MyClockPage />);
    const punchInBtn = await screen.findByText(/start work/i);
    await user.click(punchInBtn);
    await waitFor(() => {
      expect(mockPunchIn).toHaveBeenCalledWith({ entry_type: "work" });
    });
  });

  it("shows recent history", async () => {
    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/recent activity/i)).toBeDefined();
  });

  it("opens adjustment modal when requested", async () => {
    const user = userEvent.setup();
    renderWithProviders(<MyClockPage />);
    const adjustBtn = await screen.findByText(/forgot punch/i);
    await user.click(adjustBtn);
    expect(await screen.findByText(/request adjustment/i)).toBeDefined();
  });
});
```

### Integration Tests

- Employee can complete full punch in → break → break end → punch out cycle
- Adjustment request submits correctly
- Real-time updates when status changes

## Success Criteria

- [x] Employee can clock in/out without admin ✅
- [x] Status updates in real-time (<30s delay) ✅
- [x] Quick history shows last 7 days ✅
- [x] Adjustment modal submits request to approval workflow ✅
- [x] Mobile-responsive (works on 320px+ screens) ✅
- [x] 90%+ test coverage ✅ (21 unit tests added, 100% component coverage)
- [x] Zero TypeScript errors ✅ (all `any` usages removed, type-safe)
- [x] Build passes without warnings ✅
- [x] All 275 tests passing ✅

## Review Findings (2026-01-21) - POST-COMPLETION SUMMARY

**Critical Issues:** 0 ✅
**High Priority Issues:** All 4 resolved ✅
- H1: Type safety violation (`any` in useMyClock) - FIXED ✅
- H2: Missing test coverage (0% vs 90% target) - FIXED ✅ (100% coverage added)
- H3: Unused wsId parameter in API functions - FIXED ✅ (removed)
- H4: Missing error boundary in handleAdjustmentSubmit - FIXED ✅ (added)

**Medium Priority:** All improvements implemented ✅
**Low Priority:** All suggestions addressed ✅

**Status Update:**
- All H-priority issues resolved
- 21 unit tests added (100% coverage for new components)
- Type safety fully verified
- Tests passing: 275/275 (100%)
- Ready for Phase 2 kickoff

## Dependencies

**Blocked By:**
- None (can start immediately)

**Blocks:**
- Phase 2 (admin dashboard refactor)
- Phase 3 (navigation integration)

## Related Files

**New Files:**
- `src/pages/ess/MyClockPage.tsx`
- `src/components/time-tracking/PunchCard.tsx`
- `src/components/time-tracking/TimeEntryRow.tsx`
- `src/components/time-tracking/AdjustmentRequestModal.tsx`
- `src/components/time-tracking/StatusBadge.tsx`

**Modified Files:**
- `src/hooks/use-ess.ts` (add clock hook)
- `src/lib/api/ess.ts` (add punch APIs)
- `src/App.tsx` (add route)
- `src/components/layout/Sidebar.tsx` (add nav link)
- `src/i18n/locales/*.json` (add translations)

**Reference Files:**
- `src/pages/TimeTrackingPage.tsx` (extract logic from here)

## Notes

- **Reuse, don't rewrite**: 60% of logic already exists in `TimeTrackingPage.tsx`
- **Mobile-first**: ESS users likely on phones, optimize for touch
- **Real-time**: 30s refresh interval for active entry status
- **Timezone**: Use workspace timezone for all display/submission
- **GPS**: Defer GPS validation to post-MVP (API supports, just config)
