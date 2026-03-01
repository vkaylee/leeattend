import { Page } from '@playwright/test';
import { MOCK_WORKSPACE, MOCK_USER, MOCK_EMPLOYEES, MOCK_DEPARTMENTS } from './mock-data';

export async function setupApiMocks(page: Page) {
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

    // 7. Agents
    if (pathname.match(/\/api\/agents/)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify([]) });
    }

    // 8. Devices
    if (pathname.match(/\/api\/devices/)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify([]) });
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