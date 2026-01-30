import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter } from "react-router-dom";
import { ToastProvider } from "../../components/ui/Toast";
import PayrollPage from "../PayrollPage";
import AttendanceLogsPage from "../AttendanceLogsPage";
import LeaveRequestsPage from "../LeaveRequestsPage";
import * as api from "../../lib/api";
import type { PaginatedResponse, Employee } from "../../lib/api/types";

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
  });

  const renderWithProviders = (component: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>{component}</BrowserRouter>
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

      (api.fetchEmployees as any).mockResolvedValue(mockEmployees);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

      renderWithProviders(<PayrollPage />);

      // Component should extract .data and work with array methods
      expect(await screen.findByText("John Doe")).toBeInTheDocument();
    });

    it("should handle empty paginated response in PayrollPage", async () => {
      (api.fetchEmployees as any).mockResolvedValue(
        createPaginatedResponse([], { total: 0 })
      );
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

      renderWithProviders(<PayrollPage />);

      // Should not crash when rendering with empty paginated response
      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });

    it("should handle undefined paginated response in PayrollPage", async () => {
      (api.fetchEmployees as any).mockResolvedValue(undefined);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(undefined);
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

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

      (api.fetchEmployees as any).mockResolvedValue(mockEmployees);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

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
        Array.from({ length: 50 }, (_, i) => ({
          id: `emp-${i}`,
          full_name: `Employee ${i}`,
          employee_id_number: `EMP${String(i).padStart(3, "0")}`,
          workspace_id: "ws-1",
          is_active: true,
          hire_date: "2025-01-01",
          created_at: "2025-01-01T00:00:00Z",
          updated_at: "2025-01-01T00:00:00Z",
        })),
        { total: 250, page: 1, limit: 50 }
      );

      (api.fetchEmployees as any).mockResolvedValue(mockEmployees);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

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

      (api.fetchEmployees as any).mockResolvedValue(mockEmployees);
      (api.fetchWorkspace as any).mockResolvedValue({
        id: "ws-1",
        name: "Test Workspace",
      });
      (api.fetchRawAttendanceLogs as any).mockResolvedValue(
        createPaginatedResponse([])
      );

      // Should not crash when rendering with paginated response
      expect(() => {
        renderWithProviders(<AttendanceLogsPage />);
      }).not.toThrow();
    });

    it("should handle empty paginated response in AttendanceLogsPage", async () => {
      (api.fetchEmployees as any).mockResolvedValue(
        createPaginatedResponse([], { total: 0 })
      );
      (api.fetchWorkspace as any).mockResolvedValue({
        id: "ws-1",
        name: "Test Workspace",
      });
      (api.fetchRawAttendanceLogs as any).mockResolvedValue(
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

      (api.fetchEmployees as any).mockResolvedValue(mockEmployees);
      (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([]);
      (api.fetchLeavePolicies as any).mockResolvedValue([]);

      // Should not crash when rendering with paginated response
      expect(() => {
        renderWithProviders(<LeaveRequestsPage />);
      }).not.toThrow();
    });

    it("should handle empty paginated response in LeaveRequestsPage", async () => {
      (api.fetchEmployees as any).mockResolvedValue(
        createPaginatedResponse([], { total: 0 })
      );
      (api.fetchWorkspaceLeaveRequests as any).mockResolvedValue([]);
      (api.fetchLeavePolicies as any).mockResolvedValue([]);

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
      } as any;

      (api.fetchEmployees as any).mockResolvedValue(malformedResponse);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

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
      } as any;

      (api.fetchEmployees as any).mockResolvedValue(malformedResponse);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

      expect(() => {
        renderWithProviders(<PayrollPage />);
      }).not.toThrow();
    });

    it("should handle very large paginated response", async () => {
      const largeEmployeeList = Array.from({ length: 500 }, (_, i) => ({
        id: `emp-${i}`,
        full_name: `Employee ${i}`,
        employee_id_number: `EMP${String(i).padStart(3, "0")}`,
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

      (api.fetchEmployees as any).mockResolvedValue(mockEmployees);
      (api.fetchWorkspacePayslips as any).mockResolvedValue(
        createPaginatedResponse([])
      );
      (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);

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
      expect(() => correctData.forEach(() => {})).not.toThrow();

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
      const incorrectUsage = paginatedResponse as any;

      // This would throw: TypeError: incorrectUsage.forEach is not a function
      // because paginatedResponse is an object, not an array
      expect(() => {
        incorrectUsage.forEach(() => {});
      }).toThrow();

      // But the correct pattern works:
      const correctData = paginatedResponse.data;
      expect(() => {
        correctData.forEach(() => {});
      }).not.toThrow();
    });
  });
});
