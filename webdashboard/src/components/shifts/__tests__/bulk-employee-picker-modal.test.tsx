import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { BulkEmployeePickerModal } from "../bulk-employee-picker-modal";

// Identity debounce so search filtering applies synchronously in tests.
vi.mock("@/hooks/use-debounce", () => ({
  useDebounce: <T,>(value: T) => value,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params?.count !== undefined ? `${key}_${String(params.count)}` : key,
    i18n: { language: "en" },
  }),
}));

const searchEmployees = vi.fn();
vi.mock("@/lib/api", () => ({
  searchEmployees: (...args: unknown[]) => searchEmployees(...args),
}));

interface EmpRow {
  id: string;
  full_name: string;
  employee_id_number: string;
}

const ENG: EmpRow[] = [
  { id: "e1", full_name: "Alice Eng", employee_id_number: "E001" },
  { id: "e2", full_name: "Bob Eng", employee_id_number: "E002" },
  { id: "e3", full_name: "Carol Eng", employee_id_number: "E003" },
];
const SALES: EmpRow[] = [
  { id: "s1", full_name: "Dave Sales", employee_id_number: "S001" },
];

const departments = [
  { id: "d-eng", name: "Engineering", active_employee_count: ENG.length },
  { id: "d-sales", name: "Sales", active_employee_count: SALES.length },
] as never;

function renderModal(overrides: Partial<React.ComponentProps<typeof BulkEmployeePickerModal>> = {}) {
  const onClose = vi.fn();
  const onCommit = vi.fn();
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  render(
    <QueryClientProvider client={queryClient}>
      <BulkEmployeePickerModal
        departments={departments}
        initialIds={[]}
        initialMeta={[]}
        isOpen
        onClose={onClose}
        onCommit={onCommit}
        wsId="ws-1"
        {...overrides}
      />
    </QueryClientProvider>,
  );
  return { onClose, onCommit };
}

function mockDeptResponses() {
  searchEmployees.mockImplementation((params: { departmentIds?: string[] }) => {
    const deptId = params.departmentIds?.[0];
    const data = deptId === "d-eng" ? ENG : deptId === "d-sales" ? SALES : [];
    return Promise.resolve({ data, total: data.length, page: 1, limit: 50 });
  });
}

describe("BulkEmployeePickerModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockDeptResponses();
  });

  it("renders a group per department with its employee count", () => {
    renderModal();
    expect(screen.getByText("Engineering")).toBeDefined();
    expect(screen.getByText("Sales")).toBeDefined();
    expect(screen.getByText("(3)")).toBeDefined();
  });

  it("checking a department loads and selects all its employees", async () => {
    const user = userEvent.setup();
    renderModal();

    const deptCheckbox = screen.getByLabelText(
      "shifts.monthly.select_whole_department: Engineering",
    );
    await user.click(deptCheckbox);

    await waitFor(() => {
      expect(screen.getByText("shifts.monthly.bulk_selected_count_3")).toBeDefined();
    });
    expect(searchEmployees).toHaveBeenCalledWith(
      expect.objectContaining({ departmentIds: ["d-eng"], wsId: "ws-1" }),
    );
  });

  it("unchecking a department removes its employees from the draft", async () => {
    const user = userEvent.setup();
    renderModal();

    const deptCheckbox = screen.getByLabelText(
      "shifts.monthly.select_whole_department: Engineering",
    );
    await user.click(deptCheckbox);
    await waitFor(() => {
      expect(screen.getByText("shifts.monthly.bulk_selected_count_3")).toBeDefined();
    });

    await user.click(deptCheckbox);
    await waitFor(() => {
      expect(screen.getByText("shifts.monthly.bulk_selected_count_0")).toBeDefined();
    });
  });

  it("expanding a department lists individual employees and lets you toggle one", async () => {
    const user = userEvent.setup();
    renderModal();

    await user.click(screen.getByText("Engineering"));

    await waitFor(() => {
      expect(screen.getByText("Alice Eng")).toBeDefined();
    });

    await user.click(screen.getByText("Alice Eng"));
    await waitFor(() => {
      expect(screen.getByText("shifts.monthly.bulk_selected_count_1")).toBeDefined();
    });
  });

  it("search filters the employees within an expanded department", async () => {
    const user = userEvent.setup();
    renderModal();

    await user.click(screen.getByText("Engineering"));
    await waitFor(() => {
      expect(screen.getByText("Alice Eng")).toBeDefined();
    });

    await user.type(
      screen.getByPlaceholderText("shifts.monthly.bulk_search_placeholder"),
      "alice",
    );

    await waitFor(() => {
      expect(screen.queryByText("Bob Eng")).toBeNull();
    });
    expect(screen.getByText("Alice Eng")).toBeDefined();
  });

  it("Done commits the current draft ids and meta", async () => {
    const user = userEvent.setup();
    const { onCommit, onClose } = renderModal();

    await user.click(
      screen.getByLabelText("shifts.monthly.select_whole_department: Sales"),
    );
    await waitFor(() => {
      expect(screen.getByText("shifts.monthly.bulk_selected_count_1")).toBeDefined();
    });

    await user.click(screen.getByTestId("bulk-picker-done"));

    expect(onCommit).toHaveBeenCalledWith(
      ["s1"],
      expect.arrayContaining([expect.objectContaining({ id: "s1" })]),
    );
    expect(onClose).toHaveBeenCalled();
  });

  it("Cancel closes without committing", async () => {
    const user = userEvent.setup();
    const { onCommit, onClose } = renderModal();

    await user.click(
      screen.getByLabelText("shifts.monthly.select_whole_department: Sales"),
    );
    await waitFor(() => {
      expect(screen.getByText("shifts.monthly.bulk_selected_count_1")).toBeDefined();
    });

    await user.click(screen.getByText("common.cancel"));

    expect(onCommit).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("deselect all clears the draft", async () => {
    const user = userEvent.setup();
    renderModal({ initialIds: ["e1"], initialMeta: [ENG[0]] as never });

    expect(screen.getByText("shifts.monthly.bulk_selected_count_1")).toBeDefined();

    await user.click(screen.getByText("shifts.monthly.deselect_all"));
    expect(screen.getByText("shifts.monthly.bulk_selected_count_0")).toBeDefined();
  });

  it("loads every page when a department spans more than one page", async () => {
    const user = userEvent.setup();
    // 120 employees across 3 pages of 50/50/20, so fetchAllDeptEmployees must
    // walk pages until it has accumulated `total` rows.
    const BIG = Array.from({ length: 120 }, (_, i) => ({
      id: `b${String(i)}`,
      full_name: `Emp ${String(i)}`,
      employee_id_number: `B${String(i).padStart(3, "0")}`,
    }));
    searchEmployees.mockImplementation(
      (params: { departmentIds?: string[]; page?: number; limit?: number }) => {
        if (params.departmentIds?.[0] !== "d-big") {
          return Promise.resolve({ data: [], total: 0, page: 1, limit: 50 });
        }
        const limit = params.limit ?? 50;
        const page = params.page ?? 1;
        const start = (page - 1) * limit;
        return Promise.resolve({
          data: BIG.slice(start, start + limit),
          total: BIG.length,
          page,
          limit,
        });
      },
    );

    renderModal({
      departments: [
        { id: "d-big", name: "BigDept", active_employee_count: BIG.length },
      ] as never,
    });

    await user.click(
      screen.getByLabelText("shifts.monthly.select_whole_department: BigDept"),
    );

    await waitFor(() => {
      expect(
        screen.getByText("shifts.monthly.bulk_selected_count_120"),
      ).toBeDefined();
    });
    // 3 pages: 50 + 50 + 20.
    expect(searchEmployees).toHaveBeenCalledTimes(3);
  });

  it("shows the department checkbox as indeterminate on a partial selection", async () => {
    const user = userEvent.setup();
    renderModal();

    await user.click(screen.getByText("Engineering"));
    await waitFor(() => {
      expect(screen.getByText("Alice Eng")).toBeDefined();
    });

    // Select 1 of 3 → partial → indeterminate.
    await user.click(screen.getByText("Alice Eng"));

    const deptCheckbox = screen.getByLabelText(
      "shifts.monthly.select_whole_department: Engineering",
    ) as HTMLInputElement;

    await waitFor(() => {
      expect(deptCheckbox.indeterminate).toBe(true);
    });
    expect(deptCheckbox.checked).toBe(false);
  });
});
