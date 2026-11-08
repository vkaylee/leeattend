import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { EmployeeOverlays } from "../employee-overlays";

// ── Mocks ──────────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string) => k }),
}));

// Each lazy-loaded overlay renders a test-id so we can assert it loaded.
vi.mock("@/components/features/employees/employee-detail-drawer", () => ({
  default: (props: { isOpen: boolean; employee: unknown; onClose: () => void }) =>
    props.isOpen ? <div data-testid="employee-detail-drawer">Drawer</div> : null,
}));

vi.mock("@/components/features/employees/employee-form-modal", () => ({
  default: (props: { isOpen: boolean; onClose: () => void; onSuccess: () => void }) =>
    props.isOpen ? <div data-testid="employee-form-modal">Form</div> : null,
}));

vi.mock("@/components/features/attendance/attendance-import-dialog", () => ({
  default: (props: { isOpen: boolean }) =>
    props.isOpen ? <div data-testid="attendance-import-dialog">Import</div> : null,
}));

vi.mock("@/components/features/employees/employee-termination-import-dialog", () => ({
  default: (props: { isOpen: boolean }) =>
    props.isOpen ? <div data-testid="termination-import-dialog">Termination Import</div> : null,
}));

vi.mock("@/components/features/employees/transfer-employee-modal", () => ({
  default: (props: { isOpen: boolean }) =>
    props.isOpen ? <div data-testid="transfer-employee-modal">Transfer</div> : null,
}));

vi.mock("@/components/features/employees/terminate-employee-modal", () => ({
  default: (props: { isOpen: boolean }) =>
    props.isOpen ? <div data-testid="terminate-employee-modal">Terminate</div> : null,
}));

vi.mock("@/components/features/employees/rehire-employee-modal", () => ({
  default: (props: { isOpen: boolean }) =>
    props.isOpen ? <div data-testid="rehire-employee-modal">Rehire</div> : null,
}));

// ── Helpers ────────────────────────────────────────────────────────────────

const mockEmployee = {
  id: "emp-1",
  active_employment_id: "active-emp-1",
  employee_id_number: "E001",
  full_name: "Alice Nguyen",
  department_name: "Engineering",
  hire_date: "2025-01-01",
  status: "active",
} as unknown as import("@/lib/api/types").Employee;

function createDefaults(overrides: Partial<Parameters<typeof EmployeeOverlays>[0]> = {}) {
  return {
    departments: [],
    deptHistory: [],
    editingEmployee: null,
    formatDate: (d: string) => d,
    handleDrawerClose: vi.fn(),
    isImportDialogOpen: false,
    isModalOpen: false,
    isRehireModalOpen: false,
    isTerminateModalOpen: false,
    isTerminationImportOpen: false,
    isTransferModalOpen: false,
    prefillEnrollmentId: "",
    selectedDeptId: "",
    selectedEmployee: null,
    setEditingEmployee: vi.fn(),
    setIsImportDialogOpen: vi.fn(),
    setIsModalOpen: vi.fn(),
    setIsRehireModalOpen: vi.fn(),
    setIsTerminateModalOpen: vi.fn(),
    setIsTerminationImportOpen: vi.fn(),
    setIsTransferModalOpen: vi.fn(),
    setPrefillEnrollmentId: vi.fn(),
    setSelectedDeptId: vi.fn(),
    setSelectedEmployee: vi.fn(),
    ...overrides,
  };
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe("EmployeeOverlays", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("when all overlays are closed", () => {
    it("renders nothing visible", () => {
      const { container } = render(<EmployeeOverlays {...createDefaults()} />);
      // All Suspense fallbacks are null; no overlay is open.
      expect(container.textContent).toBe("");
    });
  });

  describe("EmployeeDetailDrawer (lazy-loaded)", () => {
    it("renders the drawer when selectedEmployee is set", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({ selectedEmployee: mockEmployee })}
        />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("employee-detail-drawer")).toBeDefined();
      });
    });

    it("does not render the drawer when selectedEmployee is null", () => {
      render(<EmployeeOverlays {...createDefaults({ selectedEmployee: null })} />);
      expect(screen.queryByTestId("employee-detail-drawer")).toBeNull();
    });
  });

  describe("EmployeeFormModal (lazy-loaded)", () => {
    it("renders when isModalOpen is true", async () => {
      render(<EmployeeOverlays {...createDefaults({ isModalOpen: true })} />);

      await waitFor(() => {
        expect(screen.getByTestId("employee-form-modal")).toBeDefined();
      });
    });

    it("does not render when isModalOpen is false", () => {
      render(<EmployeeOverlays {...createDefaults({ isModalOpen: false })} />);
      expect(screen.queryByTestId("employee-form-modal")).toBeNull();
    });
  });

  describe("AttendanceImportDialog (lazy-loaded)", () => {
    it("renders when isImportDialogOpen is true", async () => {
      render(
        <EmployeeOverlays {...createDefaults({ isImportDialogOpen: true })} />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("attendance-import-dialog")).toBeDefined();
      });
    });
  });

  describe("TransferEmployeeModal (lazy-loaded)", () => {
    it("renders when isTransferModalOpen is true", async () => {
      render(
        <EmployeeOverlays {...createDefaults({ isTransferModalOpen: true })} />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("transfer-employee-modal")).toBeDefined();
      });
    });
  });

  describe("TerminateEmployeeModal (lazy-loaded)", () => {
    it("renders when isTerminateModalOpen is true", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({
            isTerminateModalOpen: true,
            selectedEmployee: mockEmployee,
          })}
        />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("terminate-employee-modal")).toBeDefined();
      });
    });
  });

  describe("RehireEmployeeModal (lazy-loaded)", () => {
    it("renders when isRehireModalOpen is true", async () => {
      render(
        <EmployeeOverlays {...createDefaults({ isRehireModalOpen: true })} />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("rehire-employee-modal")).toBeDefined();
      });
    });
  });

  describe("EmployeeTerminationImportDialog (lazy-loaded)", () => {
    it("renders when isTerminationImportOpen is true", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({ isTerminationImportOpen: true })}
        />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("termination-import-dialog")).toBeDefined();
      });
    });
  });

  describe("multiple overlays", () => {
    it("can render drawer and modal together", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({
            isModalOpen: true,
            selectedEmployee: mockEmployee,
          })}
        />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("employee-detail-drawer")).toBeDefined();
        expect(screen.getByTestId("employee-form-modal")).toBeDefined();
      });
    });
  });
});
