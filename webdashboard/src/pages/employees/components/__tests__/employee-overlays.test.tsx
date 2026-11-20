import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import Button from "@/components/ui/button";

import { EmployeeOverlays } from "../employee-overlays";

// ── Mocks ──────────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string) => k }),
}));

// Each lazy-loaded overlay renders a test-id so we can assert it loaded.
vi.mock("@/components/features/employees/employee-form-modal", () => ({
  default: (props: { isOpen: boolean; onClose: () => void }) =>
    props.isOpen ? (
      <div data-testid="employee-form-modal">
        Form
        <Button data-testid="employee-form-modal-close" onClick={props.onClose} type="button" variant="ghost">Close</Button>
      </div>
    ) : null,
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
  default: (props: { isOpen: boolean; onSuccess?: () => void }) =>
    props.isOpen ? (
      <div data-testid="transfer-employee-modal">
        Transfer
        <Button data-testid="transfer-success" onClick={props.onSuccess} type="button" variant="ghost">Success</Button>
      </div>
    ) : null,
}));

vi.mock("@/components/features/employees/terminate-employee-modal", () => ({
  default: (props: { isOpen: boolean }) =>
    props.isOpen ? <div data-testid="terminate-employee-modal">Terminate</div> : null,
}));

vi.mock("@/components/features/employees/rehire-employee-modal", () => ({
  default: (props: { isOpen: boolean; onSuccess?: () => void }) =>
    props.isOpen ? (
      <div data-testid="rehire-employee-modal">
        Rehire
        <Button data-testid="rehire-success" onClick={() => props.onSuccess?.({} as never)} type="button" variant="ghost">Success</Button>
      </div>
    ) : null,
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
    activeEmploymentId: null,
    activeModal: null,
    closeModal: vi.fn(),
    selectedDeptId: "",
    setSelectedDeptId: vi.fn(),
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

  describe("EmployeeFormModal (lazy-loaded)", () => {
    it("renders only for edit modal state", async () => {
      render(<EmployeeOverlays {...createDefaults({ activeModal: { type: "edit", employee: mockEmployee } })} />);

      await waitFor(() => {
        expect(screen.getByTestId("employee-form-modal")).toBeDefined();
      });
    });

    it("closes through shared close handler", async () => {
      const closeModal = vi.fn();
      render(
        <EmployeeOverlays
          {...createDefaults({
            activeModal: { type: "edit", employee: mockEmployee },
            closeModal,
          })}
        />,
      );

      await userEvent.setup().click(await screen.findByTestId("employee-form-modal-close"));
      expect(closeModal).toHaveBeenCalledTimes(1);
    });

    it("does not render for closed or non-edit modal states", async () => {
      const { rerender } = render(<EmployeeOverlays {...createDefaults()} />);
      expect(screen.queryByTestId("employee-form-modal")).toBeNull();

      rerender(<EmployeeOverlays {...createDefaults({ activeModal: { type: "import" } })} />);
      await waitFor(() => expect(screen.queryByTestId("employee-form-modal")).toBeNull());
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

    it("refreshes employee data after transfer", async () => {
      const onEmployeeRefresh = vi.fn();
      render(<EmployeeOverlays {...createDefaults({
        activeModal: { type: "transfer", employee: mockEmployee },
        onEmployeeRefresh,
      })} />);

      await userEvent.setup().click(await screen.findByTestId("transfer-success"));
      expect(onEmployeeRefresh).toHaveBeenCalledTimes(1);
    });
  });

  describe("TerminateEmployeeModal (lazy-loaded)", () => {
    it("renders terminate modal for terminate modal state", async () => {
      render(
        <EmployeeOverlays
          {...createDefaults({
            activeModal: { type: "terminate", employee: mockEmployee },
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

    it("refreshes employee data after rehire", async () => {
      const onEmployeeRefresh = vi.fn();
      render(<EmployeeOverlays {...createDefaults({
        activeModal: { type: "rehire", employee: mockEmployee },
        onEmployeeRefresh,
      })} />);

      await userEvent.setup().click(await screen.findByTestId("rehire-success"));
      expect(onEmployeeRefresh).toHaveBeenCalledTimes(1);
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

});
