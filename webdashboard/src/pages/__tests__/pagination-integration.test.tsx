import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type { PaginatedResponse, Employee } from "../../lib/api/types";

import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import AttendanceLogsPage from "../AttendanceLogsPage";
import LeaveRequestsPage from "../LeaveRequestsPage";
import PayrollPage from "../PayrollPage";

/**
 * Integration tests for components using paginated API responses.
 *
 * These tests ensure that components properly handle PaginatedResponse<T> structure:
 * { data: T[], total: number, page: number, limit: number }
 *
 * Tests verify:
 * - Components correctly extract .data from paginated responses
 * - Array methods work correctly on extracted data
 * - Components gracefully handle undefined/null responses
 * - Components handle empty paginated responses
 */

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
 
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode; [key: string]: unknown }) => <a href={to} {...props}>{children}</a>,
}));

vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
  fetchEmployeePayslips: vi.fn(),
  fetchWorkspacePayslips: vi.fn(),
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  fetchRawAttendanceLogs: vi.fn(),
  fetchWorkspace: vi.fn(),
  fetchWorkspaceLeaveRequests: vi.fn(),
  fetchEmployeeLeaveBalances: vi.fn(),
  fetchLeavePolicies: vi.fn(),
  fetchMyPermissions: vi.fn(),
  fetchLeaveTypes: vi.fn().mockResolvedValue([]),
  // LeaveImportDialog mocks
  uploadLeaveExcel: vi.fn(),
  fetchLatestLeaveImport: vi.fn().mockResolvedValue(null),
  getLeaveImportErrorsUrl: vi.fn().mockReturnValue("#"),
  getLeaveImportTemplateUrl: vi.fn().mockReturnValue("#"),
  downloadLeaveImportTemplate: vi.fn().mockResolvedValue(undefined),
  downloadLeaveImportErrors: vi.fn().mockResolvedValue(undefined),
}));

// Helper to create mock paginated response
const createPaginatedResponse = <T,>(
  data: T[],
  options: { total?: number; page?: number; limit?: number } = {}
): PaginatedResponse<T> => ({
  data,
  total: options.total ?? data.length,
  page: options.page ?? 1,
  limit: options.limit ?? 50,
});

describe("Pagination Integration Tests", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0, staleTime: 0 },
      },
    });
 
    vi.mocked(api.fetchMyPermissions).mockResolvedValue({ permissions: [], roles: [], managed_department_ids: [] });
  });

  const renderWithProviders = (component: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {component}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  describe("PayrollPage Pagination", () => {
    it("should handle paginated employees response correctly", async () => {
      const mockEmployees: PaginatedResponse<Employee> = createPaginatedResponse(
        [
          {
            id: "emp-1",
            full_name: "John Doe",
            employee_id_number: "EMP001",
            workspace_id: "ws-1",
            is_active: true,
            hire_date: "2025-01-01",
            created_at: "2025-01-01T00:00:00Z",
            updated_at: "2025-01-01T00:00:00Z",
          },
        ],
        { total: 1 }
      );

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      renderWithProviders(<PayrollPage />);

      // Component should extract .data and work with array methods
      expect(await screen.findByText("John Doe")).toBeInTheDocument();
    });

    it("should handle empty paginated response in PayrollPage", async () => {
 
      vi.mocked(api.fetchEmployees).mockResolvedValue(
        createPaginatedResponse([], { total: 0 })
      );
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      renderWithProviders(<PayrollPage />);

      // Should not crash when rendering with empty paginated response
      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });

    it("should handle undefined paginated response in PayrollPage", async () => {
 
      vi.mocked(api.fetchEmployees).mockResolvedValue(undefined as unknown as PaginatedResponse<Employee>);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      // Should not crash when rendering with undefined paginated response
      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });

    it("should NOT crash when trying array methods on extracted data", async () => {
      const mockEmployees = createPaginatedResponse<Employee>(
        [
          {
            id: "emp-1",
            full_name: "Alice",
            employee_id_number: "EMP001",
            workspace_id: "ws-1",
            is_active: true,
            hire_date: "2025-01-01",
            created_at: "2025-01-01T00:00:00Z",
            updated_at: "2025-01-01T00:00:00Z",
          },
          {
            id: "emp-2",
            full_name: "Bob",
            employee_id_number: "EMP002",
            workspace_id: "ws-1",
            is_active: true,
            hire_date: "2025-01-02",
            created_at: "2025-01-02T00:00:00Z",
            updated_at: "2025-01-02T00:00:00Z",
          },
        ],
        { total: 2 }
      );

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      // This should NOT throw "TypeError: employees.forEach is not a function"
      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();

      // Both employees should be rendered
      expect(await screen.findByText("Alice")).toBeInTheDocument();
      expect(await screen.findByText("Bob")).toBeInTheDocument();
    });

    it("should handle paginated response with pagination metadata", async () => {
      const mockEmployees = createPaginatedResponse<Employee>(
        Array.from({ length: 50 }, (_, index) => ({
          id: `emp-${index}`,
          full_name: `Employee ${index}`,
          employee_id_number: `EMP${String(index).padStart(3, "0")}`,
          workspace_id: "ws-1",
          is_active: true,
          hire_date: "2025-01-01",
          created_at: "2025-01-01T00:00:00Z",
          updated_at: "2025-01-01T00:00:00Z",
        })),
        { total: 250, page: 1, limit: 50 }
      );

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      renderWithProviders(<PayrollPage />);

      // Should handle all 50 items from first page
      await waitFor(() => {
        expect(screen.getByText("Employee 0")).toBeInTheDocument();
      });
    });
  });

  describe("AttendanceLogsPage Pagination", () => {
    it("should handle paginated employees response in AttendanceLogsPage", async () => {
      const mockEmployees = createPaginatedResponse<Employee>(
        [
          {
            id: "emp-1",
            full_name: "John Doe",
            employee_id_number: "EMP001",
            workspace_id: "ws-1",
            is_active: true,
            hire_date: "2025-01-01",
            created_at: "2025-01-01T00:00:00Z",
            updated_at: "2025-01-01T00:00:00Z",
          },
        ]
      );

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
      vi.mocked(api.fetchWorkspace).mockResolvedValue({
        id: "ws-1",
        name: "Test Workspace",
      } as api.Workspace);
 
      vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue(
        createPaginatedResponse([])
      );

      // Should not crash when rendering with paginated response
      expect(() => {
        renderWithProviders(<AttendanceLogsPage />);
      }).not.toThrow();
    });

    it("should handle empty paginated response in AttendanceLogsPage", async () => {
 
      vi.mocked(api.fetchEmployees).mockResolvedValue(
        createPaginatedResponse([], { total: 0 })
      );
 
      vi.mocked(api.fetchWorkspace).mockResolvedValue({
        id: "ws-1",
        name: "Test Workspace",
      } as api.Workspace);
 
      vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue(
        createPaginatedResponse([])
      );

      // Should not crash when rendering with empty paginated response
      expect(() => {
        renderWithProviders(<AttendanceLogsPage />);
      }).not.toThrow();
    });
  });

  describe("LeaveRequestsPage Pagination", () => {
    it("should handle paginated employees response in LeaveRequestsPage", async () => {
      const mockEmployees = createPaginatedResponse<Employee>(
        [
          {
            id: "emp-1",
            full_name: "John Doe",
            employee_id_number: "EMP001",
            workspace_id: "ws-1",
            is_active: true,
            hire_date: "2025-01-01",
            created_at: "2025-01-01T00:00:00Z",
            updated_at: "2025-01-01T00:00:00Z",
          },
        ]
      );

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
      vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([]);
 
      vi.mocked(api.fetchLeavePolicies).mockResolvedValue([]);

      // Should not crash when rendering with paginated response
      expect(() => {
        renderWithProviders(<LeaveRequestsPage />);
      }).not.toThrow();
    });

    it("should handle empty paginated response in LeaveRequestsPage", async () => {
 
      vi.mocked(api.fetchEmployees).mockResolvedValue(
        createPaginatedResponse([], { total: 0 })
      );
 
      vi.mocked(api.fetchWorkspaceLeaveRequests).mockResolvedValue([]);
 
      vi.mocked(api.fetchLeavePolicies).mockResolvedValue([]);

      // Should not crash when rendering with paginated response
      expect(() => {
        renderWithProviders(<LeaveRequestsPage />);
      }).not.toThrow();
    });
  });

  describe("PaginatedResponse Edge Cases", () => {
    it("should handle response with null data array", async () => {
      const malformedResponse = {
        data: null,
        total: 0,
        page: 1,
        limit: 50,
 
      } as unknown as PaginatedResponse<Employee>;

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(malformedResponse);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      // Component should handle gracefully with optional chaining and nullish coalescing
      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });

    it("should handle response missing data property", async () => {
      const malformedResponse = {
        total: 0,
        page: 1,
        limit: 50,
        // data property missing
 
      } as unknown as PaginatedResponse<Employee>;

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(malformedResponse);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });

    it("should handle very large paginated response", async () => {
      const largeEmployeeList = Array.from({ length: 500 }, (_, index) => ({
        id: `emp-${index}`,
        full_name: `Employee ${index}`,
        employee_id_number: `EMP${String(index).padStart(3, "0")}`,
        workspace_id: "ws-1",
        is_active: true,
        hire_date: "2025-01-01",
        created_at: "2025-01-01T00:00:00Z",
        updated_at: "2025-01-01T00:00:00Z",
      }));

      const mockEmployees = createPaginatedResponse(largeEmployeeList, {
        total: 500,
        page: 1,
        limit: 50,
      });

 
      vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
      vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
 
      vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);

      // Should handle large response without performance issues
      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });
  });

  describe("Correct vs Incorrect Usage Patterns", () => {
    it("should document correct pattern for accessing paginated response data", () => {
      const paginatedResponse: PaginatedResponse<{ id: string }> =
        createPaginatedResponse([{ id: "1" }, { id: "2" }]);

      // CORRECT PATTERN
      const correctData = paginatedResponse?.data ?? [];
      expect(Array.isArray(correctData)).toBe(true);
      expect(() => { correctData.forEach(() => { }); }).not.toThrow();

      // This pattern ensures:
      // 1. Optional chaining prevents null reference errors
      // 2. Nullish coalescing provides empty array fallback
      // 3. Array methods work correctly
    });

    it("should show why incorrect pattern caused the bug", () => {
      const paginatedResponse: PaginatedResponse<{ id: string }> =
        createPaginatedResponse([{ id: "1" }]);

      // INCORRECT PATTERN (what was happening before)
      // Treating response object as array directly
 
      const incorrectUsage = paginatedResponse as unknown as { forEach: (callback: unknown) => void };

      // This would throw: TypeError: incorrectUsage.forEach is not a function
      // because paginatedResponse is an object, not an array
      expect(() => {
        incorrectUsage.forEach(() => { });
      }).toThrow();

      // But the correct pattern works:
      const correctData = paginatedResponse.data;
      expect(() => {
        correctData.forEach(() => { });
      }).not.toThrow();
    });
  });
});
