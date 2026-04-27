import { Page } from '@playwright/test';
import { MOCK_WORKSPACE, MOCK_USER, MOCK_EMPLOYEES, MOCK_DEPARTMENTS, MOCK_AGENTS, MOCK_DEVICES_INITIAL } from './mock-data';
import { Device } from '../../src/lib/api/types';

// --- Stateful mock stores (reset per page via setupApiMocks) ---
let devices: Device[];
let nextDeviceId = 100;

export async function setupApiMocks(page: Page) {
  // Reset stateful stores for each test
  devices = structuredClone(MOCK_DEVICES_INITIAL);
  nextDeviceId = 100;

  // Chặn các request gọi đến Backend API (/api/...)
  // Loại trừ các file tĩnh (.ts, .js, .css, .jsx, .tsx) của Vite
  await page.route(url => url.pathname.startsWith('/api/') && !url.pathname.match(/\.(ts|js|css|jsx|tsx)$/), async (route) => {
    const url = route.request().url();
    const method = route.request().method();
    const pathname = new URL(url).pathname;

    console.log(`[MOCK API] Intercepted API Call: ${method} ${pathname}`);

    const fulfill = async (data: any, status = 200) => {
      console.log(`[MOCK API] Fulfilling ${method} ${pathname} with status ${status}`);
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
    if (pathname.match(/\/api\/me(\/|$)/) || pathname.match(/\/api\/auth\/profile/)) {
      if (pathname.includes('/permissions')) {
        return fulfill({
          permissions: ["workspace.admin", "devices.manage", "agents.manage"],
          roles: ["Admin"],
          managed_department_ids: null
        });
      }
      return fulfill(MOCK_USER);
    }

    if (pathname.match(/\/api\/auth\/login/) || pathname.match(/\/api\/debug\/login/)) {
      return fulfill({
        access_token: "mock-access-token",
        refresh_token: "mock-refresh-token",
        user: MOCK_USER
      });
    }

    if (pathname.match(/\/api\/auth\/refresh/)) {
      return fulfill({
        access_token: "mock-access-token-refreshed",
        refresh_token: "mock-refresh-token-new"
      });
    }

    // 3. Workspaces
    if (pathname.match(/\/api\/workspaces/)) {
      // Handle GET /api/workspaces/{id}/preferences
      if (pathname.includes('/preferences')) {
         return fulfill({
            dashboard_layout: {},
            theme: "light",
            notifications_enabled: true
          });
      }

      // Handle Workspace members
      if (pathname.includes('/members')) {
        return fulfill([
          { id: "member-1", user_id: MOCK_USER.id, role: "admin", user: MOCK_USER }
        ]);
      }

      if (method === 'GET') {
        if (pathname === '/api/workspaces' || pathname === '/api/workspaces/') {
          return fulfill([MOCK_WORKSPACE]);
        }
        // Explicitly handle GET /api/workspaces/{id}
        const wsIdMatch = pathname.match(/\/api\/workspaces\/([^/]+)/);
        if (wsIdMatch) {
          return fulfill(MOCK_WORKSPACE);
        }
      }
      
      if (method === 'POST') {
        return fulfill(MOCK_WORKSPACE, 201);
      }

      return fulfill(MOCK_WORKSPACE);
    }

    // 4. Employees
    if (pathname.match(/\/api\/employees/)) {
      return fulfill({
        data: MOCK_EMPLOYEES,
        pagination: { total_count: MOCK_EMPLOYEES.length, page: 1, page_size: 50, total_pages: 1 }
      });
    }

    // 5. Departments
    if (pathname.match(/\/api\/departments/)) {
      return fulfill(MOCK_DEPARTMENTS);
    }

    // 6. Attendance Logs
    if (pathname.match(/\/api\/attendance\/logs/)) {
      return fulfill({
        data: [
          { id: "log-1", employee_id: "emp-mock-1", punch_time: "2026-01-15T08:00:00Z", punch_type: "in", source: "device", device_name: "Main Door" }
        ],
        pagination: { total_count: 1, page: 1, page_size: 50, total_pages: 1 }
      });
    }

    // 7. Dashboard Stats & Trends
    if (pathname.match(/\/api\/dashboard\/workspace-stats/)) {
      return fulfill({ employee_count: 2, present_today: 1, on_leave: 0, pending_requests: 0 });
    }

    if (pathname.match(/\/api\/dashboard\/attendance-trend/)) {
      return fulfill([
        { date: "2026-01-10", present: 20, absent: 2, late: 1 },
        { date: "2026-01-11", present: 22, absent: 0, late: 0 }
      ]);
    }

    if (pathname.match(/\/api\/dashboard\/unexcused-absences/)) {
      return fulfill({
        data: [],
        pagination: { total_count: 0, page: 1, page_size: 50, total_pages: 0 }
      });
    }

    if (pathname.match(/\/api\/dashboard\/overtime-trend/)) {
      return fulfill([
        { date: "2026-01-10", hours: 5.5 },
        { date: "2026-01-11", hours: 3.2 }
      ]);
    }

    if (pathname.match(/\/api\/dashboard\/peak-times/)) {
      return fulfill([
        { hour: 8, count: 15 },
        { hour: 9, count: 5 }
      ]);
    }

    if (pathname.match(/\/api\/dashboard\/leave-types/)) {
      return fulfill([
        { type: "SICK", count: 2 },
        { type: "ANNUAL", count: 5 }
      ]);
    }

    if (pathname.match(/\/api\/dashboard\/lateness-by-day/)) {
      return fulfill([
        { day: "Mon", count: 3 },
        { day: "Tue", count: 1 }
      ]);
    }

    if (pathname.match(/\/api\/dashboard\/preferences/)) {
      return fulfill({
        dashboard_layout: {},
        theme: "light",
        notifications_enabled: true
      });
    }

    if (pathname.match(/\/api\/health/)) {
      return fulfill({ status: "ok" });
    }

    // 8. Authorized Agents (READ)
    if (pathname.match(/\/api\/authorized-agents(\/|$)/)) {
      if (method === 'GET') {
        const mockAgents = MOCK_AGENTS.map(a => ({
          ...a,
          is_trusted: true,
          last_seen: new Date().toISOString()
        }));
        console.log(`[MOCK API] Fulfilling agents list with ${mockAgents.length} items`);
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
    if (pathname.match(/\/api\/agents(\/|$)/) || pathname.match(/\/api\/agents\/status/)) {
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
    if (pathname.match(/\/api\/devices(\/|$)/)) {
      // GET /api/devices
      if (method === 'GET' && (pathname === '/api/devices' || pathname === '/api/devices/')) {
        return fulfill(devices);
      }

      // POST /api/devices (create)
      if (method === 'POST' && (pathname === '/api/devices' || pathname === '/api/devices/')) {
        const body = JSON.parse(route.request().postData() || '{}');
        const newDevice: Device = {
          id: `device-new-${nextDeviceId++}`,
          agent_id: body.agent_id || "agent-mock-1",
          name: body.name || "New Device",
          ip: body.ip || "0.0.0.0",
          port: body.port || 4370,
          is_enabled: true,
          brand: body.brand || "zkteco",
          password: body.password,
          timezone: body.timezone || "UTC",
          last_status: "pending",
          created_at: new Date().toISOString(),
          workspace_id: MOCK_WORKSPACE.id,
        };
        devices.push(newDevice);
        return fulfill(newDevice, 201);
      }

      // Extract device ID from URL pattern /api/devices/{id}
      const idMatch = pathname.match(/\/api\/devices\/([^/]+)/);
      if (idMatch) {
        const deviceId = idMatch[1];

        // PUT /api/devices/:id (update)
        if (method === 'PUT') {
          const body = JSON.parse(route.request().postData() || '{}');
          const idx = devices.findIndex(d => d.id === deviceId);
          if (idx >= 0) {
            devices[idx] = { ...devices[idx], ...body };
            return fulfill(devices[idx]);
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
      return fulfill([]);
    }
    if (pathname.includes('/schedules')) {
      return fulfill({ data: [], pagination: { total_count: 0, page: 1, page_size: 50, total_pages: 0 } });
    }
    if (pathname.includes('/holidays')) {
      return fulfill([]);
    }
    if (pathname.includes('/leave/')) {
      if (pathname.endsWith('/submit')) return fulfill({ success: true }, 201);
      return fulfill({ data: [], pagination: { total_count: 0, page: 1, page_size: 50, total_pages: 0 } });
    }
    if (pathname.includes('/payroll') || pathname.includes('/salary-components')) {
      return fulfill([]);
    }
    if (pathname.includes('/assignment-rules')) {
      return fulfill([]);
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
}
