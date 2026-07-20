 
import { Page } from '@playwright/test';

import { Device } from '../../src/lib/api/types';
import { MOCK_WORKSPACE, MOCK_USER, MOCK_EMPLOYEES, MOCK_DEPARTMENTS, MOCK_AGENTS, MOCK_DEVICES_INITIAL } from './mock-data';

export async function setupApiMocks(page: Page) {
  // --- Stateful mock stores (isolated per page instance via closures) ---
  let devices = structuredClone(MOCK_DEVICES_INITIAL);
  let nextDeviceId = 100;
  let activeWorkspace = structuredClone(MOCK_WORKSPACE);
  let exportJobCalls = 0;

  // Chặn các request gọi đến Backend API (/api/...)
  // Loại trừ các file tĩnh (.ts, .js, .css, .jsx, .tsx) của Vite
  await page.route(url => url.pathname.startsWith('/api/') && !(/\.(ts|js|css|jsx|tsx)$/.exec(url.pathname)), async (route) => {
    const url = route.request().url();
    const method = route.request().method();
    let pathname = new URL(url).pathname;
    
    // Normalize path by stripping /v1/ prefix if present to support both raw and versioned routes
    pathname = pathname.replace(/^\/api\/v1\//, "/api/");

    

 
    const fulfill = async (data: unknown, status = 200) => {
      
      return route.fulfill({
        status,
        contentType: 'application/json',
        body: JSON.stringify(data)
      });
    };

    // 1. Health check
    if (pathname === '/api/health') {
      return fulfill({ status: "ok" });
    }

    // 2. Auth & Profile
    if ((/\/api\/me(\/|$)/.exec(pathname)) || (/\/api\/auth\/profile/.exec(pathname))) {
      if (pathname.includes('/permissions')) {
        return fulfill({
          permissions: [
            "workspace.admin", "devices.manage", "agents.manage",
            "workspace:update", "workspace:delete", "workspace:settings:read", "workspace:settings:write", "approval_workflows:read", "approval_workflows:manage",
            "members:read", "members:invite", "members:roles:manage", "members:remove",
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
            "agents:read", "agents:manage", "devices:read", "devices:manage", "feature_flags:read", "feature_flags:manage", "infra:monitor",
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

    // 3. Workspaces
    if (pathname === '/api/workspaces' || pathname === '/api/workspaces/') {
      if (method === 'GET') return fulfill([activeWorkspace]);
      if (method === 'POST') return fulfill(activeWorkspace, 201);
      return fulfill([activeWorkspace]);
    }
    
    // Explicitly handle GET/PATCH /api/workspaces/{id} (without sub-resources)
    if (/^\/api\/workspaces\/[^/]+$/.exec(pathname)) {
      if (method === 'GET') return fulfill(activeWorkspace);
      if (method === 'PATCH' || method === 'PUT') {
        const body = JSON.parse(route.request().postData() ?? '{}');
        activeWorkspace = { ...activeWorkspace, ...body };
        return fulfill(activeWorkspace);
      }
      return fulfill(activeWorkspace);
    }

    if (/^\/api\/workspaces\/[^/]+\/preferences$/.exec(pathname)) {
      return fulfill({ dashboard_layout: {}, theme: "light", notifications_enabled: true });
    }

    if (/^\/api\/workspaces\/[^/]+\/members$/.exec(pathname)) {
      return fulfill([ { id: "member-1", user_id: MOCK_USER.id, role: "admin", user: MOCK_USER } ]);
    }

    // 4. Employees
    if (/\/api\/(workspaces\/[^/]+\/)?employees\/?$/.exec(pathname) || /\/api\/employees\/?$/.exec(pathname)) {
      return fulfill({
        data: MOCK_EMPLOYEES,
        pagination: { total_count: MOCK_EMPLOYEES.length, page: 1, page_size: 50, totalpages: 1 }
      });
    }

    // 5. Departments
    if (/\/api\/(workspaces\/[^/]+\/)?departments\/?$/.exec(pathname) || /\/api\/departments\/?$/.exec(pathname)) {
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
          { id: "log-1", employee_id: "emp-mock-1", punch_time: "2026-01-15T08:00:00Z", punch_type: "in", source: "device", device_name: "Main Door" }
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
    if (/\/api\/dashboard\/workspace-stats/.exec(pathname)) {
      return fulfill({ employee_count: 2, present_today: 1, on_leave: 0, pending_requests: 0 });
    }

    if (/\/api\/dashboard\/attendance-trend/.exec(pathname)) {
      return fulfill([
        { date: "2026-01-10", present: 20, absent: 2, late: 1 },
        { date: "2026-01-11", present: 22, absent: 0, late: 0 }
      ]);
    }

    if (/\/api\/dashboard\/unexcused-absences/.exec(pathname)) {
      return fulfill({
        data: [],
        pagination: { total_count: 0, page: 1, page_size: 50, totalpages: 0 }
      });
    }

    if (/\/api\/dashboard\/overtime-trend/.exec(pathname)) {
      return fulfill([
        { date: "2026-01-10", hours: 5.5 },
        { date: "2026-01-11", hours: 3.2 }
      ]);
    }

    if (/\/api\/dashboard\/peak-times/.exec(pathname)) {
      return fulfill([
        { hour: 8, count: 15 },
        { hour: 9, count: 5 }
      ]);
    }

    if (/\/api\/dashboard\/leave-types/.exec(pathname)) {
      return fulfill([
        { type: "SICK", count: 2 },
        { type: "ANNUAL", count: 5 }
      ]);
    }

    if (/\/api\/dashboard\/lateness-by-day/.exec(pathname)) {
      return fulfill([
        { day: "Mon", count: 3 },
        { day: "Tue", count: 1 }
      ]);
    }

    if (/\/api\/dashboard\/preferences/.exec(pathname)) {
      return fulfill({
        dashboard_layout: {},
        theme: "light",
        notifications_enabled: true
      });
    }

    if (/\/api\/health/.exec(pathname)) {
      return fulfill({ status: "ok" });
    }

    if (pathname === '/api/system/tenants') {
      return fulfill([
        {
          id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
          name: "Mock Tenant Workspace",
          is_byod: false,
          timezone: "UTC",
          created_at: new Date().toISOString()
        }
      ]);
    }

    if (pathname === '/api/system/roles') {
      return fulfill([
        {
          id: "role-1",
          name: "platform_admin",
          description: "Full control over the system",
          assignees: []
        },
        {
          id: "role-2",
          name: "support",
          description: "Read-only access for support staff",
          assignees: []
        }
      ]);
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
          workspace_id: "ws-mock-123",
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

    // 8. Authorized Agents (READ)
    if (/\/api\/authorized-agents(\/|$)/.exec(pathname)) {
      if (method === 'GET') {
        const mockAgents = MOCK_AGENTS.map(a => ({
          ...a,
          is_trusted: true,
          last_seen: new Date().toISOString()
        }));
        
        return fulfill(mockAgents);
      }
      if (method === 'POST') {
        return fulfill({ success: true }, 201);
      }
      if (method === 'PATCH' || method === 'PUT') {
        return fulfill(MOCK_AGENTS[0]);
      }
    }

    // 9. Agent Status (realtime polling)
    if ((/\/api\/agents(\/|$)/.exec(pathname)) || (/\/api\/agents\/status/.exec(pathname))) {
      return fulfill([
        {
          agent_id: "agent-mock-1",
          is_online: true,
          last_seen: new Date().toISOString(),
          devices: devices.map(d => ({
            device_id: d.id,
            is_connected: d.is_enabled,
            employee_count: 0,
            last_sync: new Date().toISOString()
          }))
        }
      ]);
    }

    // 10. Devices — STATEFUL CRUD
    if (/\/api\/devices(\/|$)/.exec(pathname)) {
      // GET /api/devices
      if (method === 'GET' && (pathname === '/api/devices' || pathname === '/api/devices/')) {
        return fulfill(devices);
      }

      // POST /api/devices (create)
      if (method === 'POST' && (pathname === '/api/devices' || pathname === '/api/devices/')) {
        const body = JSON.parse(route.request().postData() ?? '{}');
        const newDevice: Device = {
          id: `device-new-${nextDeviceId++}`,
          agent_id: body.agent_id ?? "agent-mock-1",
          name: body.name ?? "New Device",
          ip: body.ip ?? "0.0.0.0",
          port: body.port ?? 4370,
          is_enabled: true,
          brand: body.brand ?? "zkteco",
          password: body.password,
          timezone: body.timezone ?? "UTC",
          last_status: "pending",
          created_at: new Date().toISOString(),
          workspace_id: MOCK_WORKSPACE.id,
        };
        devices.push(newDevice);
        return fulfill(newDevice, 201);
      }

      // Extract device ID from URL pattern /api/devices/{id}
      const idMatch = /\/api\/devices\/([^/]+)/.exec(pathname);
      if (idMatch) {
        const deviceId = idMatch[1];

        // PUT /api/devices/:id (update)
        if (method === 'PUT') {
          const body = JSON.parse(route.request().postData() ?? '{}');
          const index = devices.findIndex(d => d.id === deviceId);
          if (index >= 0) {
            devices[index] = { ...devices[index], ...body };
            return fulfill(devices[index]);
          }
          return fulfill({ error: "not found" }, 404);
        }

        // DELETE /api/devices/:id
        if (method === 'DELETE') {
          devices = devices.filter(d => d.id !== deviceId);
          return fulfill({ success: true });
        }
      }

      // Fallback for unhandled device routes
      return fulfill(devices);
    }

    // 11. Working Patterns, Shifts, Schedules, Holidays, Leaves (Common workspace resources)
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
    if ((/\/api\/workspaces\/[^/]+\/reports\/attendance/.exec(pathname)) && method === 'GET') {
      return fulfill({
        data: [],
        total: 0,
        page: 1,
        limit: 50
      });
    }

    if ((/\/api\/workspaces\/[^/]+\/reports\/export$/.exec(pathname)) && method === 'POST') {
      exportJobCalls = 0; // Reset counter for new job
      const resp = { job_id: "job-mock-123", total_batches: 5 };
      
      return fulfill(resp, 201);
    }
    if ((/\/api\/workspaces\/[^/]+\/reports\/jobs\/[^/]+$/.exec(pathname)) && method === 'GET') {
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

    if ((/\/api\/workspaces\/[^/]+\/reports\/jobs\/[^/]+\/download$/.exec(pathname)) && method === 'GET') {
      return route.fulfill({
        status: 200,
        contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        body: 'mock-file-content'
      });
    }

    // Mặc định trả về success cho các POST/PUT/DELETE khác để không gãy flow UI
    if (['POST', 'PUT', 'DELETE'].includes(method)) {
      return fulfill({ success: true, id: "new-id" });
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
