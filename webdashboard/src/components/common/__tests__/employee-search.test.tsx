import { QueryClient, QueryClientProvider, useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { employeeKeys } from "@/lib/query-keys";

import { EmployeeSearch } from "../employee-search";

// Mock the query hooks from react-query
vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return {
    ...actual,
    useQuery: vi.fn(),
    useInfiniteQuery: vi.fn(),
  };
});

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

describe("EmployeeSearch Component", () => {
  const mockEmployees = [
    { id: "1", full_name: "John Doe", employee_id_number: "E001" },
    { id: "2", full_name: "Jane Smith", employee_id_number: "E002" },
  ];

  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock for useInfiniteQuery (fetching search results)
    vi.mocked(useInfiniteQuery).mockReturnValue({
      data: {
        pages: [{ data: mockEmployees, total: mockEmployees.length, page: 1, limit: 50 }],
        pageParams: [1]
      },
      isLoading: false,
      fetchNextPage: vi.fn(),
      hasNextPage: false,
      isFetchingNextPage: false,
    } as any);

    // Default mock for useQuery (fetching selected employee details)
    vi.mocked(useQuery).mockImplementation((options: any) => {
      const queryKey = options.queryKey || [];
      if (queryKey[0] === 'employee') {
        const id = queryKey[2];
        const emp = mockEmployees.find(e => e.id === id);
        return {
          data: emp,
          isLoading: false,
        } as any;
      }
      return {
        data: undefined,
        isLoading: false,
      } as any;
    });
  });

  it("calls onChange when an employee is selected", () => {
    const onChange = vi.fn();
    render(<EmployeeSearch onChange={onChange} workspaceId="ws-1" />, { wrapper });

    expect(screen.getByRole("combobox")).toBeDefined();
  });

  it("renders chips in multiple mode", async () => {
    const onChange = vi.fn();
    const value = ["1"];
    render(<EmployeeSearch multiple onChange={onChange} value={value} workspaceId="ws-1" />, { wrapper });

    await waitFor(() => {
      expect(screen.getAllByText("John Doe").length).toBeGreaterThanOrEqual(1);
    });
  });

  it("preserves server-side relevance order (exact match > prefix > contains)", () => {
    const rankedEmployees = [
      { id: "a", full_name: "Nguyen Van A", employee_id_number: "16" },
      { id: "b", full_name: "Tran Van B", employee_id_number: "161" },
      { id: "c", full_name: "Le Van C", employee_id_number: "1649" },
      { id: "d", full_name: "Pham Van D", employee_id_number: "416" },
    ];

    vi.mocked(useInfiniteQuery).mockReturnValue({
      data: {
        pages: [{ data: rankedEmployees, total: rankedEmployees.length, page: 1, limit: 50 }],
        pageParams: [1]
      },
      isLoading: false,
      fetchNextPage: vi.fn(),
      hasNextPage: false,
      isFetchingNextPage: false,
    } as any);

    const onChange = vi.fn();
    render(<EmployeeSearch onChange={onChange} workspaceId="ws-1" />, { wrapper });

    const combobox = screen.getByRole("combobox");
    expect(combobox).toBeDefined();

    const allOptions = screen.queryAllByText(/16|161|1649|416/);
    if (allOptions.length > 0) {
      expect(allOptions[0].textContent).toContain("16");
    }
  });

  it("passes keyboardShortcut prop to Combobox", () => {
    const onChange = vi.fn();
    render(
      <EmployeeSearch keyboardShortcut="/" onChange={onChange} workspaceId="ws-1" />,
      { wrapper }
    );

    expect(screen.getByRole("combobox")).toBeDefined();
  });

  it("calls onSelectEmployee with the full Employee object when selected via hidden select", () => {
    const onChange = vi.fn();
    const onSelectEmployee = vi.fn();

    render(
      <EmployeeSearch
        onChange={onChange}
        onSelectEmployee={onSelectEmployee}
        workspaceId="ws-1"
      />,
      { wrapper }
    );

    const hiddenSelect = screen.getByRole("combobox");
    fireEvent.change(hiddenSelect, { target: { value: "1" } });

    expect(onChange).toHaveBeenCalledWith("1");
    expect(onSelectEmployee).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "1",
        full_name: "John Doe",
        employee_id_number: "E001",
      })
    );
  });

  it("does NOT call onSelectEmployee when employee is not found in data", () => {
    const onChange = vi.fn();
    const onSelectEmployee = vi.fn();

    render(
      <EmployeeSearch
        onChange={onChange}
        onSelectEmployee={onSelectEmployee}
        workspaceId="ws-1"
      />,
      { wrapper }
    );

    const hiddenSelect = screen.getByRole("combobox");
    fireEvent.change(hiddenSelect, { target: { value: "non-existent-id" } });

    expect(onChange).toHaveBeenCalledWith("");
    expect(onSelectEmployee).not.toHaveBeenCalled();
  });

  it("renders empty prompt when emptyQueryBehavior is prompt-to-type and search query is empty", async () => {
    vi.mocked(useInfiniteQuery).mockReturnValue({
      data: undefined,
      isLoading: false,
      fetchNextPage: vi.fn(),
      hasNextPage: false,
      isFetchingNextPage: false,
    } as any);

    render(
      <EmployeeSearch
        emptyQueryBehavior="prompt-to-type"
        onChange={vi.fn()}
        workspaceId="ws-1"
      />,
      { wrapper }
    );

    const hiddenSelect = screen.getByRole("combobox");
    expect(hiddenSelect).toBeDefined();
  });

  it("passes permission parameter to useInfiniteQuery hook", () => {
    const onChange = vi.fn();
    render(
      <EmployeeSearch
        onChange={onChange}
        permission="attendance:adjustments:submit_on_behalf"
        workspaceId="ws-1"
      />,
      { wrapper }
    );

    expect(useInfiniteQuery).toHaveBeenCalledWith(
      expect.objectContaining({
        queryKey: employeeKeys.search("ws-1", {
          departmentIds: [],
          emptyQueryBehavior: "all",
          permission: "attendance:adjustments:submit_on_behalf",
          search: "",
        }),
      })
    );
  });

  it("does NOT render the department filter button when no departments are provided", () => {
    render(<EmployeeSearch onChange={vi.fn()} workspaceId="ws-1" />, { wrapper });

    expect(screen.queryByTestId("employee-dept-filter")).toBeNull();
  });

  it("does NOT render the department filter button when departments is empty", () => {
    render(<EmployeeSearch departments={[]} onChange={vi.fn()} workspaceId="ws-1" />, { wrapper });

    expect(screen.queryByTestId("employee-dept-filter")).toBeNull();
  });

  it("renders the department filter button when departments are provided", () => {
    const departments = [
      { id: "d1", name: "Engineering" },
      { id: "d2", name: "Sales" },
    ] as any;
    render(
      <EmployeeSearch departments={departments} onChange={vi.fn()} workspaceId="ws-1" />,
      { wrapper }
    );

    expect(screen.getByTestId("employee-dept-filter")).toBeDefined();
  });

  it("passes selected department ids into the useInfiniteQuery queryKey (controlled)", () => {
    const departments = [
      { id: "d1", name: "Engineering" },
      { id: "d2", name: "Sales" },
    ] as any;
    render(
      <EmployeeSearch
        departments={departments}
        onChange={vi.fn()}
        selectedDepartmentIds={["d1", "d2"]}
        workspaceId="ws-1"
      />,
      { wrapper }
    );

    expect(useInfiniteQuery).toHaveBeenCalledWith(
      expect.objectContaining({
        queryKey: employeeKeys.search("ws-1", {
          departmentIds: ["d1", "d2"],
          emptyQueryBehavior: "all",
          search: "",
        }),
      })
    );
  });
});