# Phase 2: Admin Dashboard Refactor

**Priority:** P1
**Complexity:** Medium
**Duration:** 6-8 hours
**Status:** Pending

## Overview

Refactor `/time-tracking` page from "select employee → punch them in/out" to "team oversight dashboard". Admins view real-time team status, initiate adjustments, approve adjustment requests, and access analytics.

## Objectives

1. Real-time team status grid (who's in/out/break)
2. Admin-initiated adjustments (fix errors, missed punches)
3. Approve/reject employee adjustment requests
4. Analytics summary (total hours, late arrivals, early departures)
5. Export functionality

## Technical Approach

### Page Structure

**Before (Current):**
```
/time-tracking
├─ Employee selector
├─ Punch card (admin punches for employee)
└─ History table
```

**After (Refactored):**
```
/time-tracking (Admin Dashboard)
├─ Team Status Grid (real-time who's in/out/break)
├─ Analytics Summary Cards
├─ Adjustment Requests (pending approvals)
├─ Admin Adjustment Modal (override punch times)
└─ Export Actions
```

## Implementation Steps

### Step 1: Redesign Page Layout (2h)

**File:** `src/pages/TimeTrackingPage.tsx` (REFACTOR existing file)

**New Structure:**
```typescript
export default function TimeTrackingPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();

  // State
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string>("");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [isAdjustModalOpen, setIsAdjustModalOpen] = useState(false);

  // Data
  const { data: employees = [] } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId!),
    enabled: !!wsId,
  });

  // Fetch all team entries for today
  const { data: teamEntries = [], isLoading } = useQuery({
    queryKey: ["team-time-entries", wsId, format(selectedDate, 'yyyy-MM-dd')],
    queryFn: () => fetchTeamTimeEntries(wsId!, format(selectedDate, 'yyyy-MM-dd')),
    enabled: !!wsId,
    refetchInterval: 30000, // Refresh every 30s
  });

  // Fetch pending adjustment requests
  const { data: pendingAdjustments = [] } = useQuery({
    queryKey: ["attendance-adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId!, { status: "pending" }),
    enabled: !!wsId,
  });

  // Calculate analytics
  const analytics = useMemo(() => {
    const currentlyWorking = teamEntries.filter(e => !e.punch_out_time && e.entry_type === "work").length;
    const onBreak = teamEntries.filter(e => !e.punch_out_time && e.entry_type === "break").length;
    const offDuty = employees.length - currentlyWorking - onBreak;

    return { currentlyWorking, onBreak, offDuty, total: employees.length };
  }, [teamEntries, employees]);

  if (!wsId) return <div>{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      {/* Header with Analytics Cards */}
      <div className="flex flex-col gap-8">
        <div className="flex items-center justify-between px-2">
          <div>
            <h1 className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">
              {t("time_tracking.admin.title")}
            </h1>
            <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60 mt-1.5">
              {t("time_tracking.admin.subtitle")}
            </p>
          </div>

          <div className="flex gap-4">
            <button className="flex items-center gap-2 px-6 py-3 bg-white border border-slate-200 rounded-2xl text-xs font-black uppercase tracking-widest hover:bg-slate-50">
              <Download size={16} />
              {t("common.export")}
            </button>
            <button
              onClick={() => setViewMode(viewMode === "grid" ? "list" : "grid")}
              className="p-3 bg-white border border-slate-200 rounded-2xl hover:bg-slate-50"
            >
              {viewMode === "grid" ? <List size={20} /> : <Grid size={20} />}
            </button>
          </div>
        </div>

        {/* Analytics Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <AnalyticsCard
            label={t("time_tracking.admin.currently_working")}
            value={analytics.currentlyWorking}
            total={analytics.total}
            icon={<Users className="text-primary-600" />}
            color="primary"
          />
          <AnalyticsCard
            label={t("time_tracking.admin.on_break")}
            value={analytics.onBreak}
            total={analytics.total}
            icon={<Coffee className="text-amber-600" />}
            color="amber"
          />
          <AnalyticsCard
            label={t("time_tracking.admin.off_duty")}
            value={analytics.offDuty}
            total={analytics.total}
            icon={<Home className="text-slate-600" />}
            color="slate"
          />
          <AnalyticsCard
            label={t("time_tracking.admin.pending_adjustments")}
            value={pendingAdjustments.length}
            icon={<AlertCircle className="text-rose-600" />}
            color="rose"
          />
        </div>
      </div>

      {/* Date Selector */}
      <div className="flex items-center gap-4 bg-white p-6 rounded-[2rem] border border-slate-200 shadow-sm">
        <Calendar className="text-slate-400" size={20} />
        <input
          type="date"
          value={format(selectedDate, 'yyyy-MM-dd')}
          onChange={(e) => setSelectedDate(new Date(e.target.value))}
          className="flex-1 bg-transparent border-none text-sm font-black uppercase tracking-tight outline-none"
        />
      </div>

      {/* Team Status View */}
      {viewMode === "grid" ? (
        <TeamStatusGrid
          employees={employees}
          teamEntries={teamEntries}
          isLoading={isLoading}
          onAdminAdjust={(employeeId) => {
            setSelectedEmployeeId(employeeId);
            setIsAdjustModalOpen(true);
          }}
        />
      ) : (
        <TeamStatusList
          employees={employees}
          teamEntries={teamEntries}
          isLoading={isLoading}
          onAdminAdjust={(employeeId) => {
            setSelectedEmployeeId(employeeId);
            setIsAdjustModalOpen(true);
          }}
        />
      )}

      {/* Pending Adjustment Requests */}
      {pendingAdjustments.length > 0 && (
        <PendingAdjustmentsSection
          adjustments={pendingAdjustments}
          employees={employees}
        />
      )}

      {/* Admin Adjustment Modal */}
      <AdminAdjustmentModal
        isOpen={isAdjustModalOpen}
        onClose={() => setIsAdjustModalOpen(false)}
        employeeId={selectedEmployeeId}
        employees={employees}
      />
    </div>
  );
}
```

### Step 2: Build Team Status Grid Component (2h)

**File:** `src/components/time-tracking/TeamStatusGrid.tsx`

```typescript
interface TeamStatusGridProps {
  employees: Employee[];
  teamEntries: TimeEntry[];
  isLoading: boolean;
  onAdminAdjust: (employeeId: string) => void;
}

export function TeamStatusGrid({ employees, teamEntries, isLoading, onAdminAdjust }: TeamStatusGridProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  // Build map of employee ID → active entry
  const activeEntriesMap = useMemo(() => {
    const map = new Map<string, TimeEntry>();
    teamEntries.forEach(entry => {
      if (!entry.punch_out_time) {
        map.set(entry.employee_id, entry);
      }
    });
    return map;
  }, [teamEntries]);

  if (isLoading) {
    return <GridSkeleton count={12} />;
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {employees.map(employee => {
        const activeEntry = activeEntriesMap.get(employee.id);
        const status = activeEntry
          ? (activeEntry.entry_type === "break" ? "break" : "working")
          : "off_duty";

        return (
          <div
            key={employee.id}
            className="bg-white border border-slate-200 rounded-[2rem] p-6 shadow-sm hover:shadow-md transition-all group relative overflow-hidden"
          >
            {/* Status Badge (Top Right) */}
            <div className="absolute top-4 right-4">
              <StatusBadge status={status} />
            </div>

            {/* Employee Info */}
            <div className="flex items-center gap-4 mb-4">
              <div className="w-12 h-12 rounded-full bg-slate-100 flex items-center justify-center text-slate-600 font-black">
                {employee.first_name?.[0]}{employee.last_name?.[0]}
              </div>
              <div className="flex-1">
                <h3 className="text-sm font-black text-slate-900 uppercase tracking-tight">
                  {employee.first_name} {employee.last_name}
                </h3>
                <p className="text-[10px] text-slate-400 uppercase tracking-widest font-black">
                  {employee.employee_id_number || "N/A"}
                </p>
              </div>
            </div>

            {/* Clock Info */}
            {activeEntry ? (
              <div className="space-y-2 mt-4 pt-4 border-t border-slate-100">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-400 font-black uppercase tracking-widest">
                    {activeEntry.entry_type === "break" ? t("time_tracking.break_start") : t("time_tracking.shift_start")}
                  </span>
                  <span className="font-mono font-black text-slate-900">
                    {formatTime(activeEntry.punch_in_time)}
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-400 font-black uppercase tracking-widest">
                    {t("time_tracking.duration")}
                  </span>
                  <span className="font-mono font-black text-primary-600">
                    <LiveDuration startTime={activeEntry.punch_in_time} />
                  </span>
                </div>
              </div>
            ) : (
              <div className="text-center text-slate-400 text-xs font-black uppercase tracking-widest py-4 italic">
                {t("time_tracking.status.off_duty")}
              </div>
            )}

            {/* Admin Actions */}
            <button
              onClick={() => onAdminAdjust(employee.id)}
              className="w-full mt-4 py-2 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-xl text-[10px] font-black uppercase tracking-widest text-slate-600 transition-all opacity-0 group-hover:opacity-100"
            >
              {t("time_tracking.admin.adjust")}
            </button>
          </div>
        );
      })}
    </div>
  );
}
```

### Step 3: Build Team Status List Component (1h)

**File:** `src/components/time-tracking/TeamStatusList.tsx`

```typescript
// Alternative table/list view for admins who prefer dense layout
export function TeamStatusList({ employees, teamEntries, isLoading, onAdminAdjust }: TeamStatusGridProps) {
  // Similar to grid but table format
  // Columns: Employee Name | ID | Status | Clock In | Duration | Actions
}
```

### Step 4: Build Analytics Card Component (30min)

**File:** `src/components/time-tracking/AnalyticsCard.tsx`

```typescript
interface AnalyticsCardProps {
  label: string;
  value: number;
  total?: number;
  icon: React.ReactNode;
  color: "primary" | "amber" | "slate" | "rose";
}

export function AnalyticsCard({ label, value, total, icon, color }: AnalyticsCardProps) {
  const percentage = total ? Math.round((value / total) * 100) : 0;

  const colorClasses = {
    primary: "bg-primary-50 border-primary-100 text-primary-600",
    amber: "bg-amber-50 border-amber-100 text-amber-600",
    slate: "bg-slate-50 border-slate-100 text-slate-600",
    rose: "bg-rose-50 border-rose-100 text-rose-600",
  };

  return (
    <div className="bg-white border border-slate-200 rounded-[2rem] p-6 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <div className={`w-12 h-12 rounded-[1rem] flex items-center justify-center border ${colorClasses[color]}`}>
          {icon}
        </div>
        {total && (
          <span className="text-xs text-slate-400 font-black">
            {percentage}%
          </span>
        )}
      </div>
      <div className="text-3xl font-black text-slate-900 tracking-tight mb-1">
        {value}
        {total && <span className="text-lg text-slate-400">/{total}</span>}
      </div>
      <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">
        {label}
      </p>
    </div>
  );
}
```

### Step 5: Build Admin Adjustment Modal (2h)

**File:** `src/components/time-tracking/AdminAdjustmentModal.tsx`

```typescript
interface AdminAdjustmentModalProps {
  isOpen: boolean;
  onClose: () => void;
  employeeId: string;
  employees: Employee[];
}

export function AdminAdjustmentModal({ isOpen, onClose, employeeId, employees }: AdminAdjustmentModalProps) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const toast = useToast();

  const employee = employees.find(e => e.id === employeeId);

  const [adjustmentData, setAdjustmentData] = useState({
    date: format(new Date(), 'yyyy-MM-dd'),
    punchIn: "08:00",
    punchOut: "17:00",
    reason: "",
    isBreakOvertime: false,
  });

  const adjustMutation = useMutation({
    mutationFn: (data: any) => submitAttendanceAdjustment(wsId!, {
      ...data,
      employee_id: employeeId,
      status: "approved", // Admin adjustments auto-approved
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["team-time-entries"] });
      onClose();
      toast.success(t("time_tracking.admin.adjustment_success"));
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const tz = "UTC"; // TODO: Get workspace timezone
    const punchInTime = toUTCISO(adjustmentData.date, adjustmentData.punchIn, tz);
    const punchOutTime = toUTCISO(adjustmentData.date, adjustmentData.punchOut, tz);

    adjustMutation.mutate({
      requested_punch_in_time: punchInTime,
      requested_punch_out_time: punchOutTime,
      is_break_overtime: adjustmentData.isBreakOvertime,
      reason: adjustmentData.reason,
    });
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("time_tracking.admin.adjust_for", { name: employee?.first_name })}
      subtitle={t("time_tracking.admin.adjust_subtitle")}
      icon={<Edit size={24} />}
      maxWidth="md"
    >
      <form onSubmit={handleSubmit} className="space-y-6 pt-4">
        {/* Date, Punch In, Punch Out inputs */}
        {/* Same as AdjustmentRequestModal but admin can directly approve */}
        <button type="submit" className="w-full py-4 bg-primary-600 text-white rounded-2xl font-black uppercase tracking-widest hover:bg-primary-700">
          {t("time_tracking.admin.apply_adjustment")}
        </button>
      </form>
    </Modal>
  );
}
```

### Step 6: Build Pending Adjustments Section (1h)

**File:** `src/components/time-tracking/PendingAdjustmentsSection.tsx`

```typescript
export function PendingAdjustmentsSection({ adjustments, employees }: Props) {
  // List of pending adjustment requests
  // Admin can approve/reject each
  // Link to /attendance-adjustments for full view
}
```

### Step 7: Add Team API Endpoint (1h)

**File:** `src/lib/api/time-tracking.ts`

```typescript
export async function fetchTeamTimeEntries(wsId: string, date: string): Promise<TimeEntry[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time-entries?date=${date}`);
  if (!response.ok) throw new Error("Failed to fetch team entries");
  return response.json();
}
```

**Note:** If backend doesn't have team endpoint, aggregate from existing:
```typescript
export async function fetchTeamTimeEntries(wsId: string, date: string): Promise<TimeEntry[]> {
  const employees = await fetchEmployees(wsId);
  const allEntries = await Promise.all(
    employees.map(emp =>
      fetchTimeHistory(wsId, emp.id, date, date).catch(() => [])
    )
  );
  return allEntries.flat();
}
```

## Testing Requirements

### Unit Tests

**File:** `src/pages/__tests__/TimeTrackingPage.test.tsx` (UPDATE existing tests)

```typescript
describe("TimeTrackingPage (Admin Dashboard)", () => {
  it("renders team status grid", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/team status/i)).toBeDefined();
  });

  it("shows analytics cards", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/currently working/i)).toBeDefined();
    expect(await screen.findByText(/on break/i)).toBeDefined();
  });

  it("switches between grid and list view", async () => {
    const user = userEvent.setup();
    renderWithProviders(<TimeTrackingPage />);
    const viewToggle = await screen.findByRole("button", { name: /view/i });
    await user.click(viewToggle);
    expect(screen.getByRole("table")).toBeDefined();
  });

  it("opens admin adjustment modal", async () => {
    const user = userEvent.setup();
    renderWithProviders(<TimeTrackingPage />);
    const adjustBtn = await screen.findByText(/adjust/i);
    await user.click(adjustBtn);
    expect(await screen.findByText(/adjust for/i)).toBeDefined();
  });
});
```

## Success Criteria

- [ ] Real-time team status display (<30s refresh)
- [ ] Analytics cards show accurate counts
- [ ] Admin can initiate adjustments for any employee
- [ ] Pending adjustments section linked to approval page
- [ ] Export functionality works
- [ ] Grid/list view toggle functional
- [ ] 90%+ test coverage
- [ ] No performance issues with 100+ employees

## Dependencies

**Blocked By:**
- Phase 1 (for reusable components)

**Blocks:**
- Phase 3 (navigation integration)

## Related Files

**Modified Files:**
- `src/pages/TimeTrackingPage.tsx` (MAJOR REFACTOR)

**New Files:**
- `src/components/time-tracking/TeamStatusGrid.tsx`
- `src/components/time-tracking/TeamStatusList.tsx`
- `src/components/time-tracking/AnalyticsCard.tsx`
- `src/components/time-tracking/AdminAdjustmentModal.tsx`
- `src/components/time-tracking/PendingAdjustmentsSection.tsx`
- `src/components/time-tracking/LiveDuration.tsx` (utility for live timer)
- `src/lib/api/time-tracking.ts` (new team APIs)

**Dependencies:**
- Reuse `PunchCard`, `TimeEntryRow`, `StatusBadge` from Phase 1

## Notes

- **Real-time updates:** 30s polling for team status
- **Performance:** Virtualize grid if >100 employees
- **Export:** CSV format with date, employee, in, out, duration
- **Permissions:** Check admin role before allowing adjustments
- **Backward compat:** Keep existing adjustment approval workflow intact
