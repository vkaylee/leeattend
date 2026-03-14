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

  // Chỉ chặn các request thực sự gọi đến Backend API
  // Loại trừ các file tĩnh (.ts, .js, .css, .jsx, .tsx) của Vite
  await page.route(url => url.pathname.startsWith('/api/') && !url.pathname.match(/\.(ts|js|css|jsx|tsx)$/), async (route) => {
    const url = route.request().url();
    const method = route.request().method();
    const pathname = new URL(url).pathname;

    console.log(`[MOCK API] Intercepted API Call: ${method} ${pathname}`);

    // 1. Auth & Profile
    if (pathname.match(/\/api\/auth\/profile/) || pathname.match(/\/api\/me/)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_USER) });
    }

    if (pathname.match(/\/api\/debug\/login/) || pathname.match(/\/api\/auth\/login/)) {
      return route.fulfill({
        status: 200, contentType: 'application/json', body: JSON.stringify({
          access_token: "mock-access-token",
          refresh_token: "mock-refresh-token",
          user: MOCK_USER
        })
      });
    }

    // 2. Workspaces
    if (pathname.match(/\/api\/workspaces/)) {
      if (method === 'GET' && (pathname === '/api/workspaces' || pathname === '/api/workspaces/')) {
        return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify([MOCK_WORKSPACE]) });
      }
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_WORKSPACE) });
    }

    // 3. Employees
    if (pathname.match(/\/api\/employees/)) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          data: MOCK_EMPLOYEES,
          pagination: { total_count: MOCK_EMPLOYEES.length, page: 1, page_size: 50, total_pages: 1 }
        })
      });
    }

    // 4. Departments
    if (pathname.match(/\/api\/departments/)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_DEPARTMENTS) });
    }

    // 5. Attendance Logs
    if (pathname.match(/\/api\/attendance\/logs/)) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          data: [
            { id: "log-1", employee_id: "emp-mock-1", punch_time: "2026-01-15T08:00:00Z", punch_type: "in", source: "device", device_name: "Main Door" }
          ],
          pagination: { total_count: 1, page: 1, page_size: 50, total_pages: 1 }
        })
      });
    }

    // 6. Dashboard Stats
    if (pathname.match(/\/api\/dashboard\/workspace-stats/)) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ employee_count: 2, present_today: 1, on_leave: 0, pending_requests: 0 })
      });
    }

    // 7. Authorized Agents (READ)
    if (pathname.match(/\/api\/agents/) && !pathname.match(/\/api\/agents\/status/)) {
      if (method === 'GET') {
        return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_AGENTS) });
      }
      // POST agent command
      if (method === 'POST') {
        return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: true }) });
      }
    }

    // 8. Agent Status (realtime polling)
    if (pathname.match(/\/api\/agents\/status/)) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
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
        ])
      });
    }

    // 9. Devices — STATEFUL CRUD
    if (pathname.match(/\/api\/devices/)) {
      // GET /api/devices
      if (method === 'GET' && (pathname === '/api/devices' || pathname === '/api/devices/')) {
        return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(devices) });
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
        return route.fulfill({ status: 201, contentType: 'application/json', body: JSON.stringify(newDevice) });
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
            return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(devices[idx]) });
          }
          return route.fulfill({ status: 404, contentType: 'application/json', body: JSON.stringify({ error: "not found" }) });
        }

        // DELETE /api/devices/:id
        if (method === 'DELETE') {
          devices = devices.filter(d => d.id !== deviceId);
          return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: true }) });
        }
      }

      // Fallback for unhandled device routes
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(devices) });
    }

    // Mặc định trả về success cho các POST/PUT/DELETE khác để không gãy flow UI
    if (['POST', 'PUT', 'DELETE'].includes(method)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: true, id: "new-id" }) });
    }

    // Fallback cho các API khác - Luôn trả về Array rỗng nếu không biết, 
    // vì hầu hết logic UI của bạn duyệt qua list (employees, logs, agents...)
    console.warn(`[MOCK API] UNHANDLED ENDPOINT: ${pathname}. Returning fallback [].`);
    return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify([]) });
  });
}