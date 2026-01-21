# Phase 3: Navigation & Integration

**Priority:** P2
**Complexity:** Low
**Duration:** 2-3 hours
**Status:** Pending

## Overview

Integrate new ESS clock page and refactored admin dashboard into app navigation, routing, and i18n. Ensure role-based visibility and smooth user experience.

## Objectives

1. Add ESS clock to sidebar navigation
2. Update routes in App.tsx
3. Add role-based navigation visibility
4. Complete i18n translations
5. Add breadcrumb navigation
6. Update dashboard quick links

## Implementation Steps

### Step 1: Update Sidebar Navigation (1h)

**File:** `src/components/layout/Sidebar.tsx`

**Current ESS Section:**
```typescript
{
  section: t("common.ess"),
  items: [
    { to: "/ess/attendance", icon: CalendarDays, label: t("ess.attendance.title") }
  ]
}
```

**Update to:**
```typescript
{
  section: t("common.ess"),
  items: [
    { to: "/ess/clock", icon: Clock, label: t("ess.clock.title") }, // NEW
    { to: "/ess/attendance", icon: CalendarDays, label: t("ess.attendance.title") },
    { to: "/ess/profile", icon: User, label: t("ess.profile.title") }
  ]
}
```

**Admin Section (Ensure time-tracking visible for admins only):**
```typescript
{
  section: t("common.time_and_attendance"),
  items: [
    { to: "/time-tracking", icon: Clock, label: t("time_tracking.admin.title"), adminOnly: true }, // Add adminOnly flag
    { to: "/attendance-logs", icon: CalendarDays, label: t("common.attendance_logs"), adminOnly: true },
    { to: "/attendance-adjustments", icon: AlertCircle, label: t("common.adjustments"), adminOnly: true }
  ]
}
```

**Add Role Check Logic:**
```typescript
// Inside Sidebar component
const { user } = useAuth(); // Assuming useAuth hook provides user role

// Filter items based on role
const filteredItems = section.items.filter(item => {
  if (item.adminOnly) {
    return user?.role === "admin" || user?.role === "owner";
  }
  return true;
});
```

### Step 2: Update App Routes (30min)

**File:** `src/App.tsx`

**Add ESS Clock Route:**
```typescript
{/* ESS Routes */}
<Route path="/ess/clock" element={<MyClockPage />} />
<Route path="/ess/attendance" element={<MyAttendancePage />} />
<Route path="/ess/profile" element={<MyProfilePage />} />
```

**Ensure Admin Route Protected:**
```typescript
{/* Admin Time Tracking Route */}
<Route path="/time-tracking" element={
  <ProtectedRoute requiredRole="admin">
    <TimeTrackingPage />
  </ProtectedRoute>
} />
```

**Add ProtectedRoute Component** (if not exists):
```typescript
function ProtectedRoute({ children, requiredRole }: { children: ReactNode; requiredRole?: string }) {
  const { user } = useAuth();

  if (requiredRole && user?.role !== requiredRole && user?.role !== "owner") {
    return <Navigate to="/ess/clock" replace />;
  }

  return <>{children}</>;
}
```

### Step 3: Add i18n Translations (1h)

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
      "recent_history": "Recent Activity (Last 7 Days)",
      "forgot_punch": "Forgot to punch?",
      "request_adjustment": "Request Adjustment"
    }
  },
  "time_tracking": {
    "admin": {
      "title": "Team Time Tracking",
      "subtitle": "Monitor team attendance and manage adjustments",
      "currently_working": "Currently Working",
      "on_break": "On Break",
      "off_duty": "Off Duty",
      "pending_adjustments": "Pending Adjustments",
      "adjust": "Adjust",
      "adjust_for": "Adjust for {{name}}",
      "adjust_subtitle": "Admin adjustment (auto-approved)",
      "adjustment_success": "Adjustment applied successfully",
      "apply_adjustment": "Apply Adjustment",
      "break_start": "Break Started",
      "shift_start": "Shift Started",
      "duration": "Duration"
    },
    "status": {
      "working": "Working",
      "on_break": "On Break",
      "off_duty": "Off Duty"
    }
  }
}
```

**File:** `src/i18n/locales/vi.json` (Vietnamese)

```json
{
  "ess": {
    "clock": {
      "title": "Đồng Hồ Chấm Công",
      "subtitle": "Chấm công vào ra và nghỉ giải lao",
      "punch_success": "Chấm công thành công",
      "punch_out_success": "Chấm công ra thành công",
      "adjustment_requested": "Yêu cầu điều chỉnh đã gửi",
      "no_entries": "Không có bản ghi chấm công gần đây",
      "recent_history": "Hoạt Động Gần Đây (7 Ngày)",
      "forgot_punch": "Quên chấm công?",
      "request_adjustment": "Yêu Cầu Điều Chỉnh"
    }
  },
  "time_tracking": {
    "admin": {
      "title": "Quản Lý Chấm Công Nhóm",
      "subtitle": "Giám sát chấm công và quản lý điều chỉnh",
      "currently_working": "Đang Làm Việc",
      "on_break": "Đang Nghỉ",
      "off_duty": "Chưa Chấm Công",
      "pending_adjustments": "Yêu Cầu Chờ Duyệt",
      "adjust": "Điều Chỉnh",
      "adjust_for": "Điều chỉnh cho {{name}}",
      "adjust_subtitle": "Điều chỉnh quản trị (tự động duyệt)",
      "adjustment_success": "Điều chỉnh thành công",
      "apply_adjustment": "Áp Dụng Điều Chỉnh"
    }
  }
}
```

**File:** `src/i18n/locales/zh.json` (Chinese)

```json
{
  "ess": {
    "clock": {
      "title": "打卡钟",
      "subtitle": "上下班和休息打卡",
      "punch_success": "打卡成功",
      "punch_out_success": "下班打卡成功",
      "adjustment_requested": "调整请求已提交",
      "no_entries": "没有最近的打卡记录",
      "recent_history": "最近活动（最近7天）",
      "forgot_punch": "忘记打卡？",
      "request_adjustment": "请求调整"
    }
  },
  "time_tracking": {
    "admin": {
      "title": "团队考勤管理",
      "subtitle": "监控团队出勤并管理调整",
      "currently_working": "正在工作",
      "on_break": "休息中",
      "off_duty": "未打卡",
      "pending_adjustments": "待审批调整",
      "adjust": "调整",
      "adjust_for": "为{{name}}调整",
      "adjust_subtitle": "管理员调整（自动批准）",
      "adjustment_success": "调整成功",
      "apply_adjustment": "应用调整"
    }
  }
}
```

### Step 4: Add Breadcrumb Navigation (30min)

**File:** `src/components/layout/Breadcrumbs.tsx` (if exists, update; if not, create)

```typescript
export function Breadcrumbs() {
  const location = useLocation();
  const { t } = useTranslation();

  const breadcrumbMap: Record<string, string> = {
    "/ess/clock": t("ess.clock.title"),
    "/ess/attendance": t("ess.attendance.title"),
    "/ess/profile": t("ess.profile.title"),
    "/time-tracking": t("time_tracking.admin.title"),
    "/attendance-logs": t("common.attendance_logs"),
    "/attendance-adjustments": t("common.adjustments"),
  };

  const pathSegments = location.pathname.split("/").filter(Boolean);
  const breadcrumbs = pathSegments.map((segment, index) => {
    const path = "/" + pathSegments.slice(0, index + 1).join("/");
    return { path, label: breadcrumbMap[path] || segment };
  });

  return (
    <nav className="flex items-center gap-2 text-xs text-slate-400 mb-4">
      <Link to="/" className="hover:text-slate-600">{t("common.home")}</Link>
      {breadcrumbs.map((crumb, i) => (
        <React.Fragment key={crumb.path}>
          <ChevronRight size={14} />
          <Link
            to={crumb.path}
            className={cn(
              "hover:text-slate-600",
              i === breadcrumbs.length - 1 && "text-slate-900 font-bold"
            )}
          >
            {crumb.label}
          </Link>
        </React.Fragment>
      ))}
    </nav>
  );
}
```

**Add to Layout:**
```typescript
// In MainLayout.tsx or each page
<Breadcrumbs />
```

### Step 5: Update Dashboard Quick Links (30min)

**File:** `src/pages/DashboardPage.tsx`

**Add ESS Quick Link for Employees:**
```typescript
{/* Quick Actions for ESS */}
{user?.role === "employee" && (
  <div className="bg-white p-6 rounded-[2rem] border border-slate-200 shadow-sm">
    <h3 className="text-sm font-black uppercase tracking-widest text-slate-800 mb-4">
      {t("dashboard.quick_actions")}
    </h3>
    <div className="flex flex-col gap-3">
      <Link
        to="/ess/clock"
        className="flex items-center gap-3 p-4 bg-primary-50 border border-primary-100 rounded-xl hover:bg-primary-100 transition-all"
      >
        <Clock className="text-primary-600" size={20} />
        <span className="font-black text-xs uppercase tracking-widest text-primary-600">
          {t("ess.clock.title")}
        </span>
      </Link>
      <Link
        to="/ess/attendance"
        className="flex items-center gap-3 p-4 bg-slate-50 border border-slate-100 rounded-xl hover:bg-slate-100 transition-all"
      >
        <CalendarDays className="text-slate-600" size={20} />
        <span className="font-black text-xs uppercase tracking-widest text-slate-600">
          {t("ess.attendance.title")}
        </span>
      </Link>
    </div>
  </div>
)}

{/* Admin Quick Links */}
{(user?.role === "admin" || user?.role === "owner") && (
  <div className="bg-white p-6 rounded-[2rem] border border-slate-200 shadow-sm">
    <h3 className="text-sm font-black uppercase tracking-widest text-slate-800 mb-4">
      {t("dashboard.admin_tools")}
    </h3>
    <div className="flex flex-col gap-3">
      <Link
        to="/time-tracking"
        className="flex items-center gap-3 p-4 bg-primary-50 border border-primary-100 rounded-xl hover:bg-primary-100 transition-all"
      >
        <Users className="text-primary-600" size={20} />
        <span className="font-black text-xs uppercase tracking-widest text-primary-600">
          {t("time_tracking.admin.title")}
        </span>
      </Link>
    </div>
  </div>
)}
```

## Testing Requirements

### Integration Tests

**File:** `src/__tests__/navigation.test.tsx`

```typescript
describe("Navigation Integration", () => {
  it("shows ESS clock link for employees", async () => {
    renderWithAuth(<Sidebar />, { role: "employee" });
    expect(await screen.findByText(/time clock/i)).toBeDefined();
  });

  it("shows admin time tracking link for admins", async () => {
    renderWithAuth(<Sidebar />, { role: "admin" });
    expect(await screen.findByText(/team time tracking/i)).toBeDefined();
  });

  it("hides admin links from employees", async () => {
    renderWithAuth(<Sidebar />, { role: "employee" });
    expect(screen.queryByText(/team time tracking/i)).toBeNull();
  });

  it("navigates to ESS clock from dashboard", async () => {
    const user = userEvent.setup();
    renderWithAuth(<DashboardPage />, { role: "employee" });
    const clockLink = await screen.findByText(/time clock/i);
    await user.click(clockLink);
    expect(window.location.pathname).toBe("/ess/clock");
  });
});
```

## Success Criteria

- [ ] ESS clock link visible in sidebar for all users
- [ ] Admin time tracking link visible only for admins
- [ ] Routes navigate correctly
- [ ] Breadcrumbs show on all pages
- [ ] Dashboard quick links functional
- [ ] All translations complete (en, vi, zh)
- [ ] Role-based visibility working
- [ ] No broken links

## Dependencies

**Blocked By:**
- Phase 1 (ESS clock page)
- Phase 2 (admin dashboard)

**Blocks:**
- Phase 4 (testing)

## Related Files

**Modified Files:**
- `src/components/layout/Sidebar.tsx`
- `src/App.tsx`
- `src/pages/DashboardPage.tsx`
- `src/i18n/locales/en.json`
- `src/i18n/locales/vi.json`
- `src/i18n/locales/zh.json`

**New Files:**
- `src/components/layout/Breadcrumbs.tsx` (if not exists)
- `src/components/ProtectedRoute.tsx` (if not exists)

## Notes

- **Role Check:** Use existing auth system role checks
- **Translations:** Coordinate with i18n team if they own translation files
- **Dashboard:** Consider A/B testing quick link placement
- **Mobile:** Ensure sidebar collapsible on mobile still shows ESS links prominently
