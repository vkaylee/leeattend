 
import { Page } from '@playwright/test';
import { MOCK_ORGANIZATION, MOCK_USER, MOCK_EMPLOYEES, MOCK_DEPARTMENTS } from './mock-data';

export async function setupApiMocks(page: Page) {
  // --- Stateful mock stores (isolated per page instance via closures) ---
  let activeOrganization = structuredClone(MOCK_ORGANIZATION);
  let exportJobCalls = 0;
  const dashboardWidgetIds = new Set([
    "kpi_cards", "donut_chart", "quick_stats", "attendance_trend", "department_chart",
    "overtime_trend", "peak_times", "leave_types", "lateness_by_day", "recruitment_sources",
    "turnover_analytics", "unexcused_absences", "promo_cards",
  ]);
  const dashboardViews = new Set(["overview", "attendance", "trends", "custom"]);
  let dashboardPreferences = {
    id: "dashboard-pref-mock-1",
    user_id: MOCK_USER.id,
    active_view: "overview",
    visible_widgets: {
      kpi_cards: true, donut_chart: true, quick_stats: true, attendance_trend: true,
      department_chart: true, overtime_trend: false, peak_times: false, leave_types: false,
      lateness_by_day: false, recruitment_sources: true, turnover_analytics: false,
      unexcused_absences: false, promo_cards: false,
    },
    widget_order: [
      "kpi_cards", "donut_chart", "quick_stats", "attendance_trend", "department_chart",
      "overtime_trend", "peak_times", "leave_types", "lateness_by_day", "recruitment_sources",
      "turnover_analytics", "unexcused_absences", "promo_cards",
    ],
    collapsed_sections: { summary: false, charts: false, actions: true },
  };
  const executiveOverview = {
    contract_version: "executive-dashboard.v2",
    organization: { timezone: "Asia/Ho_Chi_Minh", currency: "VND" },
    context: {
      period: {
        key: "this_month",
        start_date: "2026-01-01",
        end_date: "2026-01-31",
        timezone: "Asia/Ho_Chi_Minh",
        as_of: "2026-01-31T00:00:00Z",
      },
      comparison: "previous_period",
      filters: {},
      scope: { kind: "organization", department_ids: [], location_ids: [], omitted_dimensions: [] },
    },
    metrics: [{
      key: "system_ot_hours",
      value: 5.5,
      unit: "hours",
      numerator: null,
      denominator: null,
      status: "available",
      health: "ok",
      null_reason: null,
      definition_version: "1",
      direction: "neutral",
      freshness: { as_of: "2026-01-31T00:00:00Z", lag_seconds: 0 },
      quality_flags: [],
      comparison: {
        basis: "none",
        period_start: "2026-01-01",
        period_end: "2026-01-31",
        value: null,
        delta: null,
        delta_percent: null,
        delta_percentage_points: null,
        status: "not_applicable",
      },
      target: null,
    }, {
      key: "approved_ot_hours",
      value: 4,
      unit: "hours",
      numerator: null,
      denominator: null,
      status: "available",
      health: "ok",
      null_reason: null,
      definition_version: "1",
      direction: "neutral",
      freshness: { as_of: "2026-01-31T00:00:00Z", lag_seconds: 0 },
      quality_flags: [],
      comparison: null,
      target: null,
    }, {
      key: "unapproved_ot_hours",
      value: 1.5,
      unit: "hours",
      numerator: null,
      denominator: null,
      status: "available",
      health: "ok",
      null_reason: null,
      definition_version: "1",
      direction: "neutral",
      freshness: { as_of: "2026-01-31T00:00:00Z", lag_seconds: 0 },
      quality_flags: [],
      comparison: null,
      target: null,
    }],
    insights: [],
    actions: { total: 0, overdue: 0, by_type: [] },
    data_quality: { status: "complete", sources: [] },
    generated_at: "2026-01-31T00:00:00Z",
  };
  const recruitmentEffectiveness = {
    period_start: "2026-01-01",
    period_end: "2026-01-31",
    total_hires: 1,
    overview: {
      hires: 1,
      average_cost_per_hire: "500000",
      cost_available: true,
      retained_90d: 1,
      retention_90d: "100",
      turnover_in_range: 0,
      comparison_hires: 1,
      comparison_hires_delta: 0,
    },
    funnel: [],
    by_source: [{
      source_id: "source-1",
      source_type: "referral",
      source_name: "Referral",
      hires: 1,
      hires_ratio: "100",
      retained_90d: 1,
      retention_90d: "100",
      cost_per_hire: "500000",
      cost_per_hire_available: true,
      turnover_in_range: 0,
    }],
    hires_trend: [{ month: "2026-01-01", hires: 1 }],
    data_gaps: [],
    warnings: [],
  };

  // Chặn các request gọi đến Backend API (/api/...)
  // Loại trừ các file tĩnh (.ts, .js, .css, .jsx, .tsx) của Vite
  await page.route(url => url.pathname.startsWith('/api/') && !(/\.(ts|js|css|jsx|tsx)$/.exec(url.pathname)), async (route) => {
    const url = route.request().url();
    const method = route.request().method();
    let pathname = new URL(url).pathname;
    
    if (pathname === "/api/v1") {
      pathname = "/api";
    } else {
      pathname = pathname.replace(/^\/api\/v1\//, "/api/");
    }

    const fulfill = async (data?: unknown, status = 200) => {
      if (data === undefined) {
        return route.fulfill({ status });
      }

      return route.fulfill({
        status,
        contentType: 'application/json',
        body: JSON.stringify(data),
      });
    };

    // Runtime feature flags: enable ESS for authenticated E2E route coverage; keep identity match off by default.
    if (pathname === '/api/feature-flags/effective') {
      return fulfill([
        { key: 'ff_ess', enabled: true },
        { key: 'ff_econtracts', enabled: true },
        { key: 'ff_employee_identity_match', enabled: false },
      ]);
    }

    // 1. Health check
    if (pathname === '/api/health') {
      return fulfill({ status: "ok" });
    }

    // 2. Auth & Profile
    if ((/\/api\/me(\/|$)/.exec(pathname)) || (/\/api\/auth\/profile/.exec(pathname))) {
      if (pathname.includes('/permissions')) {
        return fulfill({
          permissions: [
            "organization:settings:read", "organization:settings:write", "members:read", "members:invite", "members:roles:manage", "members:remove",
            "employees:read", "employees:create", "employees:update", "employees:delete", "employees:rehire", "employees:upload", "employees:bulk_edit",
            "departments:read", "departments:create", "departments:update", "departments:delete", "departments:transfer",
            "attendance:logs:read", "attendance:logs:import", "attendance:summaries:read", "attendance:adjustments:submit", "attendance:adjustments:approve", "attendance:adjustments:submit_on_behalf", "attendance:reprocess", "attendance:staging:manage", "attendance:force_clock_out",
            "overtime:claim", "overtime:approve", "overtime:policies:manage", "overtime:paper:upload", "overtime:request:create", "overtime:request:read", "overtime:request:create_on_behalf", "overtime:claim_on_behalf", "overtime:approve:workflow", "overtime:quota:read",
            "leave:requests:read", "leave:requests:submit", "leave:requests:submit_on_behalf", "leave:requests:approve", "leave:approve:l1", "leave:approve:l2", "leave:balances:read", "leave:balances:adjust", "leave:types:manage", "leave:policies:manage", "leave:accrual:run",
            "shifts:read", "shifts:create", "shifts:update", "shifts:assign", "shift_swaps:manage", "shift_swaps:approve", "patterns:read", "patterns:create", "patterns:assign", "patterns:delete", "rosters:read", "rosters:manage", "assignment_rules:manage",
            "payroll:read", "payroll:calculate", "payroll:settings:manage", "payroll:components:manage", "payroll:incentives:manage", "payroll:approve", "payroll:disburse", "payroll:sensitive:read",
            "holidays:read", "holidays:manage",
            "reports:read", "reports:export", "dashboard:read", "dashboard:preferences",
            "documents:medical:read", "documents:contract:read",
            "audit_logs:read", "audit_logs:export", "security:settings:manage",
            "recruitment:read", "recruitment:manage",
            "employment_phases:read", "employment_phases:manage",
            "ess:profile", "ess:attendance", "ess:clock", "ess:leave", "ess:payslips", "ess:shift_swaps"
          ],
          roles: ["Admin"],
          managed_department_ids: null
        });
      }
      return fulfill(MOCK_USER);
    }

    if ((/\/api\/auth\/login/.exec(pathname)) || (/\/api\/debug\/login/.exec(pathname))) {
      return fulfill({
        access_token: "mock-access-token",
        refresh_token: "mock-refresh-token",
        user: MOCK_USER
      });
    }

    if (/\/api\/auth\/refresh/.exec(pathname)) {
      return fulfill({
        access_token: "mock-access-token-refreshed",
        refresh_token: "mock-refresh-token-new"
      });
    }

    if (pathname === '/api') {
      if (method === 'GET') return fulfill(structuredClone(activeOrganization));
      if (method === 'PATCH' || method === 'PUT') {
        const body = JSON.parse(route.request().postData() ?? '{}');
        activeOrganization = { ...activeOrganization, ...body };
        return fulfill(structuredClone(activeOrganization));
      }
    }

    if (pathname === '/api/members') {
      return fulfill({
        data: [ { id: "member-1", user_id: MOCK_USER.id, role: "admin", user: MOCK_USER } ],
        total: 1,
        page: 1,
        limit: 50,
      });
    }


    // 4. Employees
    if (/\/api\/employees\/?$/.exec(pathname)) {
      return fulfill({
        data: MOCK_EMPLOYEES,
        pagination: { total_count: MOCK_EMPLOYEES.length, page: 1, page_size: 50, totalpages: 1 }
      });
    }

    // 5. Departments
    if (/\/api\/departments\/?$/.exec(pathname)) {
      return fulfill({
        data: MOCK_DEPARTMENTS,
        total: MOCK_DEPARTMENTS.length,
        page: 1,
        limit: 50
      });
    }

    // 6. Attendance Logs
    if (/\/api\/attendance\/logs/.exec(pathname)) {
      return fulfill({
        data: [
        ],
        pagination: { total_count: 1, page: 1, page_size: 50, totalpages: 1 }
      });
    }

    // 6.5. Attendance Summaries
    if (/\/attendance\/summaries/.exec(pathname)) {
      return fulfill({
        data: [],
        total: 0,
        page: 1,
        limit: 20
      });
    }

    // 7. Dashboard Stats & Trends
    if (/\/api\/dashboard\/organization-stats/.exec(pathname)) {
      return fulfill({
        absent_today: 2,
        avg_work_hours_today: 7.5,
        early_departure_today: 0,
        estimated_monthly_payroll: "125000.00",
        late_today: 1,
        on_leave_today: 0,
        pending_leave_requests: 0,
        present_today: 20,
        total_employees: 22,
        attendance_rate: { value: "90.9", numerator: 20, denominator: 22, unit: "employees" },
        absence_rate: { value: "9.1", numerator: 2, denominator: 22, unit: "employees" },
        late_rate: { value: "4.5", numerator: 1, denominator: 22, unit: "employees" },
      });
    }

    if (/\/api\/dashboard\/attendance-trend/.exec(pathname)) {
      return fulfill([
        { date: "2026-01-10", present: 20, absent: 2, late: 1, on_leave: 0, eligible: 22, attendance_rate: { value: "90.9", numerator: 20, denominator: 22, unit: "employees" }, absence_rate: { value: "9.1", numerator: 2, denominator: 22, unit: "employees" }, late_rate: { value: "4.5", numerator: 1, denominator: 22, unit: "employees" } },
        { date: "2026-01-11", present: 22, absent: 0, late: 0, on_leave: 1, eligible: 23, attendance_rate: { value: "95.7", numerator: 22, denominator: 23, unit: "employees" }, absence_rate: { value: "0", numerator: 0, denominator: 23, unit: "employees" }, late_rate: { value: "0", numerator: 0, denominator: 23, unit: "employees" } },
      ]);
    }

    if (/\/api\/dashboard\/department-attendance/.exec(pathname)) {
      return fulfill([{
        absent: 1,
        department_id: "dept-1",
        department_name: "Kỹ thuật",
        late: 2,
        on_leave: 0,
        on_time: 19,
        total_employees: 22,
        attendance_rate: { value: "86.4", numerator: 19, denominator: 22, unit: "employees" },
        absence_rate: { value: "4.5", numerator: 1, denominator: 22, unit: "employees" },
        late_rate: { value: "9.1", numerator: 2, denominator: 22, unit: "employees" },
      }]);
    }
    if (/\/api\/dashboard\/overtime-breakdown/.exec(pathname)) {
      return fulfill({
        contract_version: "overtime-breakdown.v2",
        organization: { timezone: "Asia/Ho_Chi_Minh", currency: "VND" },
        rows: [{
          department_id: "dept-1",
          department_name: "Kỹ thuật",
          system_ot_hours: 5.5,
          approved_ot_hours: 4,
          unapproved_ot_hours: 1.5,
          overtime_cost: "500000",
          cost_available: true,
        }],
        generated_at: "2026-01-31T00:00:00Z",
      });
    }

    if (pathname === '/api/employers' && method === 'GET') {
      return fulfill({ data: [], total: 0, page: 1, limit: 50 });
    }

    if (pathname === '/api/reports/attendance' && method === 'GET') {
      return fulfill({ data: [], total: 0, page: 1, limit: 50 });
    }

    if (pathname === '/api/reports/export' && method === 'POST') {
      exportJobCalls = 0;
      return fulfill({ job_id: "job-mock-123", total_batches: 5 }, 201);
    }

    if (/^\/api\/reports\/jobs\/[^/]+$/.exec(pathname) && method === 'GET') {
      exportJobCalls++;
      return fulfill({
        job_id: "job-mock-123",
        status: exportJobCalls < 3 ? "processing" : "completed",
        progress: {
          total_batches: 5,
          completed_batches: exportJobCalls < 3 ? exportJobCalls * 2 : 5,
          failed_batches: 0,
          percentage: exportJobCalls < 3 ? (exportJobCalls * 2 / 5) * 100 : 100,
        },
        created_at: new Date().toISOString(),
      });
    }

    if (/^\/api\/reports\/jobs\/[^/]+\/download$/.exec(pathname) && method === 'GET') {
      return route.fulfill({ status: 200, contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', body: 'mock-file-content' });
    }

    if (pathname === '/api/employers' && ['POST', 'PUT', 'DELETE'].includes(method)) {
      return fulfill({ error: "unhandled endpoint" }, 404);
    }

    // 7. Dashboard Stats & Trends

    if (/\/api\/dashboard\/unexcused-absences/.exec(pathname)) {
      return fulfill([]);
    }

    if (/\/api\/dashboard\/overtime-trend/.exec(pathname)) {
      return fulfill([
        { date: "2026-01-10", system_ot_hours: 5.5, approved_ot_hours: 4 },
        { date: "2026-01-11", system_ot_hours: 3.2, approved_ot_hours: 2.5 },
      ]);
    }

    if (/\/api\/dashboard\/peak-times/.exec(pathname)) {
      return fulfill([
        { time_slot: "08:00", check_in: 15, check_out: 2 },
        { time_slot: "09:00", check_in: 5, check_out: 1 },
      ]);
    }

    if (/\/api\/dashboard\/leave-types/.exec(pathname)) {
      return fulfill([
        { leave_type: "SICK", count: 2 },
        { leave_type: "ANNUAL", count: 5 },
      ]);
    }

    if (/\/api\/dashboard\/lateness-by-day/.exec(pathname)) {
      return fulfill([
        { day: "Mon", late_count: 3, avg_late_minutes: 12 },
        { day: "Tue", late_count: 1, avg_late_minutes: 8 },
      ]);
    }

    if (/\/api\/dashboard\/recruitment-sources/.exec(pathname)) {
      return fulfill([
        { source_name: "Referral", source_type: "referral", count: 4 },
        { source_name: "Job board", source_type: "job_board", count: 2 },
      ]);
    }

    if (/\/api\/dashboard\/turnover-analytics/.exec(pathname)) {
      return fulfill({
        by_department: [{ department_name: "Kỹ thuật", count: 1 }],
        by_source: [{ source_name: "Referral", source_type: "referral", count: 1 }],
        period_start: "2026-01-01",
        period_end: "2026-01-31",
        tenure_distribution: [{ bucket: "0-1 year", count: 1 }],
        total_terminated: 1,
      });
    }
    if (/\/api\/dashboard\/executive-overview/.exec(pathname)) {
      return fulfill(executiveOverview);
    }

    if (/\/api\/dashboard\/recruitment-effectiveness/.exec(pathname)) {
      return fulfill(recruitmentEffectiveness);
    }

    if (/\/api\/dashboard\/preferences/.exec(pathname)) {
      if (method === 'GET') return fulfill(structuredClone(dashboardPreferences));
      if (method !== 'PUT') return fulfill({ error: "method_not_allowed" }, 405);

      let body: unknown;
      try {
        body = JSON.parse(route.request().postData() ?? '{}');
      } catch {
        return fulfill({ error: "invalid_json" }, 400);
      }

      if (!body || typeof body !== 'object' || Array.isArray(body)) {
        return fulfill({ error: "invalid_preferences_body" }, 400);
      }

      const update = body as Record<string, unknown>;
      const allowedKeys = new Set(["active_view", "visible_widgets", "widget_order", "collapsed_sections"]);
      if (Object.keys(update).some((key) => !allowedKeys.has(key))) {
        return fulfill({ error: "unknown_preferences_field" }, 400);
      }
      if ("active_view" in update && (typeof update.active_view !== 'string' || !dashboardViews.has(update.active_view))) {
        return fulfill({ error: "invalid_active_view" }, 400);
      }
      if ("visible_widgets" in update && (
        !update.visible_widgets || typeof update.visible_widgets !== 'object' || Array.isArray(update.visible_widgets) ||
        Object.entries(update.visible_widgets).some(([id, enabled]) => !dashboardWidgetIds.has(id) || typeof enabled !== 'boolean')
      )) {
        return fulfill({ error: "invalid_visible_widgets" }, 400);
      }
      if ("widget_order" in update && (
        !Array.isArray(update.widget_order) ||
        update.widget_order.some((id) => typeof id !== 'string' || !dashboardWidgetIds.has(id))
      )) {
        return fulfill({ error: "invalid_widget_order" }, 400);
      }
      if ("collapsed_sections" in update && (
        !update.collapsed_sections || typeof update.collapsed_sections !== 'object' || Array.isArray(update.collapsed_sections) ||
        Object.values(update.collapsed_sections).some((collapsed) => typeof collapsed !== 'boolean')
      )) {
        return fulfill({ error: "invalid_collapsed_sections" }, 400);
      }

      dashboardPreferences = {
        ...dashboardPreferences,
        active_view: "active_view" in update
          ? update.active_view as string
          : dashboardPreferences.active_view,
        visible_widgets: "visible_widgets" in update
          ? { ...dashboardPreferences.visible_widgets, ...(update.visible_widgets as Record<string, boolean>) }
          : dashboardPreferences.visible_widgets,
        widget_order: "widget_order" in update
          ? [...(update.widget_order as string[])]
          : dashboardPreferences.widget_order,
        collapsed_sections: "collapsed_sections" in update
          ? { ...dashboardPreferences.collapsed_sections, ...(update.collapsed_sections as Record<string, boolean>) }
          : dashboardPreferences.collapsed_sections,
      };
      return fulfill(structuredClone(dashboardPreferences));
    }

    if (/\/api\/health/.exec(pathname)) {
      return fulfill({ status: "ok" });
    }


    if (/\/api\/notifications\/unread-count/.exec(pathname)) {
      return fulfill({ count: 0 });
    }

    if (/\/api\/notifications/.exec(pathname)) {
      return fulfill({
        next_cursor: null,
        notifications: [],
        unread_count: 0
      });
    }

    // ESS & Overtime Mocks
    if (pathname === '/api/ess/profile') {
      return fulfill({
        employee: MOCK_EMPLOYEES[0],
        emergency_contacts: []
      });
    }

    if (pathname.includes('/overtime/assignments')) {
      return fulfill([]);
    }

    if (pathname.includes('/overtime/quota')) {
      return fulfill({
        employee_id: "emp-mock-1",
        monthly_limit: 1200,
        monthly_used: 300,
        yearly_limit: 12000,
        yearly_used: 1500
      });
    }

    if (pathname.includes('/behalf-assignments/my-targets')) {
      return fulfill(MOCK_EMPLOYEES);
    }

    if (pathname.includes('/overtime/requests')) {
      if (method === 'GET') {
        return fulfill({
          data: [],
          total: 0,
          page: 1,
          limit: 20,
        });
      }
      if (method === 'POST') {
        return fulfill({
          id: "ot-req-new",
          status: "draft",
          planned_date: new Date().toISOString().split('T')[0],
          planned_minutes: 120,
          reason: "E2E Test Overtime Requirement",
          requested_by_user_id: "user-mock-1",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          employees: []
        }, 201);
      }
    }


    // 11. Working Patterns, Shifts, Schedules, Holidays, Leaves (Common organization resources)
    if (pathname.includes('/working-patterns')) {
      return fulfill([]);
    }
    if (pathname.includes('/shifts')) {
      return fulfill({
        data: [],
        total: 0,
        page: 1,
        limit: 50
      });
    }
    if (pathname.includes('/schedules')) {
      return fulfill({
        data: [],
        total: 0,
        page: 1,
        limit: 50
      });
    }
    if (pathname.includes('/assignment-history')) {
      return fulfill({
        data: [],
        total: 0,
        page: 1,
        limit: 50
      });
    }
    if (pathname.includes('/holidays')) {
      return fulfill([]);
    }
    if (pathname.includes('/leave/')) {
      if (pathname.endsWith('/submit')) return fulfill({ success: true }, 201);
      return fulfill({ data: [], pagination: { total_count: 0, page: 1, page_size: 50, totalpages: 0 } });
    }
    if (pathname.includes('/payroll') || pathname.includes('/salary-components')) {
      return fulfill([]);
    }
    if (pathname.includes('/assignment-rules')) {
      return fulfill([]);
    }

    // 12. Report Export
    if (pathname === '/api/reports/attendance' && method === 'GET') {
      return fulfill({
        data: [],
        total: 0,
        page: 1,
        limit: 50
      });
    }

    if (pathname === '/api/reports/export' && method === 'POST') {
      exportJobCalls = 0; // Reset counter for new job
      const resp = { job_id: "job-mock-123", total_batches: 5 };
      
      return fulfill(resp, 201);
    }
    if (/^\/api\/reports\/jobs\/[^/]+$/.exec(pathname) && method === 'GET') {
      exportJobCalls++;
      if (exportJobCalls < 3) {
        return fulfill({
          job_id: "job-mock-123",
          status: "processing",
          progress: { total_batches: 5, completed_batches: exportJobCalls * 2, failed_batches: 0, percentage: (exportJobCalls * 2 / 5) * 100 },
          created_at: new Date().toISOString()
        });
      }
      
      // Simulate completed job
      return fulfill({
        job_id: "job-mock-123",
        status: "completed",
        progress: { total_batches: 5, completed_batches: 5, failed_batches: 0, percentage: 100 },
        created_at: new Date().toISOString()
      });
    }

    if ((/^\/api\/reports\/jobs\/[^/]+\/download$/.exec(pathname)) && method === 'GET') {
      return route.fulfill({
        status: 200,
        contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        body: 'mock-file-content'
      });
    }

    // Keep unknown mutations visible to tests instead of masking a missing
    // endpoint with a successful response.
    if (['POST', 'PUT', 'DELETE'].includes(method)) {
      return fulfill({ error: "unhandled endpoint" }, 404);
    }

    // Fallback cho các API khác - Luôn trả về Array rỗng nếu không biết, 
    // vì hầu hết logic UI của bạn duyệt qua list (employees, logs, agents...)
    console.warn(`[MOCK API] UNHANDLED ENDPOINT: ${pathname}. Returning fallback [].`);
    return fulfill([]);
  });

  // Chặn WebSocket calls để tránh hang (mock stream API)
  await page.routeWebSocket(/.*/, ws => {
    
    if (ws.url().includes('/ws/reports/')) {
      // Simulate connection rejected so it falls back to HTTP polling
      ws.onMessage(message => {
        if (message === '{"type":"ping"}') {
          ws.send('{"type":"pong"}');
        }
      });
      // Triggers onError in useReportStream
      ws.send('{"type":"ERROR", "message": "Connection lost for mock testing"}');
      setTimeout(() => {
        void ws.close();
      }, 100);
    }
  });
}
