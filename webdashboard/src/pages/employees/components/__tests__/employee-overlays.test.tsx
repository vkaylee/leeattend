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
  default: (props: { isOpen: boolean; onClose: () => void }) =>
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
    activeEmploymentId: null,
    activeModal: null,
    closeModal: vi.fn(),
    formatDate: (d: string) => d,
    handleDrawerClose: vi.fn(),
    openModal: vi.fn(),
    selectedDeptId: "",
    selectedEmployee: null,
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
    it("renders only for edit modal state", async () => {
      render(<EmployeeOverlays {...createDefaults({ activeModal: { type: "edit", employee: mockEmployee } })} />);

      await waitFor(() => {
        expect(screen.getByTestId("employee-form-modal")).toBeDefined();
      });
    });

    it("does not render for closed modal state", () => {
      render(<EmployeeOverlays {...createDefaults()} />);
      expect(screen.queryByTestId("employee-form-modal")).toBeNull();
    });
  });

  describe("AttendanceImportDialog (lazy-loaded)", () => {
    it("renders import dialog for import modal state", async () => {
      render(
        <EmployeeOverlays {...createDefaults({ activeModal: { type: "import" } })} />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("attendance-import-dialog")).toBeDefined();
      });
    });
  });

  describe("TransferEmployeeModal (lazy-loaded)", () => {
    it("renders transfer modal for transfer modal state", async () => {
      render(
        <EmployeeOverlays {...createDefaults({ activeModal: { type: "transfer", employee: mockEmployee } })} />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("transfer-employee-modal")).toBeDefined();
      });
    });
  });

  describe("TerminateEmployeeModal (lazy-loaded)", () => {
    it("renders terminate modal for terminate modal state", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({
            activeModal: { type: "terminate", employee: mockEmployee },
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
    it("renders rehire modal for rehire modal state", async () => {
      render(
        <EmployeeOverlays {...createDefaults({ activeModal: { type: "rehire", employee: mockEmployee } })} />,
      );

      await waitFor(() => {
        expect(screen.getByTestId("rehire-employee-modal")).toBeDefined();
      });
    });
  });

  describe("EmployeeTerminationImportDialog (lazy-loaded)", () => {
    it("renders termination import for import-termination modal state", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({ activeModal: { type: "import-termination" } })}
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
            activeModal: { type: "edit", employee: mockEmployee },
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
