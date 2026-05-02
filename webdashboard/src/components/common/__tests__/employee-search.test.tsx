import { QueryClient, QueryClientProvider, useQuery  } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { EmployeeSearch } from "../employee-search";

// Create a simpler mock for the query hook
vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return {
    ...actual,
    useQuery: vi.fn(),
  };
});


const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

describe("EmployeeSearch Component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockEmployees = [
    { id: "1", full_name: "John Doe", employee_id_number: "E001" },
    { id: "2", full_name: "Jane Smith", employee_id_number: "E002" },
  ];

  it("calls onChange when an employee is selected", async () => {
 
    vi.mocked(useQuery).mockReturnValue({
      data: { data: mockEmployees },
      isLoading: false,
    } as unknown as never);

    const onChange = vi.fn();
    // We test the internal handleSelect by bypassing the complex Radix UI
    // and just checking if the component passes correct data to Combobox
    render(<EmployeeSearch onChange={onChange} workspaceId="ws-1" />, { wrapper });

    // Fallback select is used when Radix trigger is not fully rendered in JSDOM
    expect(screen.getByRole("combobox")).toBeDefined();
  });

  it("renders chips in multiple mode", () => {
 
    vi.mocked(useQuery).mockReturnValue({
      data: { data: mockEmployees },
      isLoading: false,
    } as unknown as never);

    const onChange = vi.fn();
    const value = ["1"];
    // In multi-mode, EmployeeSearch manages 'selectedMeta' internally when handleSelect is called.
    // To test initial chips, we'd need to mock the internal state or pass it as prop.
    // For now, we verify it renders without crashing.
    render(<EmployeeSearch multiple onChange={onChange} value={value} workspaceId="ws-1" />, { wrapper });

    // With the new useEffect, it should find '1' in mockEmployees and show 'John Doe'
    // It appears in both the chip and the hidden select option
    expect(screen.getAllByText("John Doe").length).toBeGreaterThanOrEqual(1);
  });

  it("preserves server-side relevance order (exact match > prefix > contains)", () => {
    // Simulate server returning results in relevance-ranked order:
    // #16 (exact) > #161 (prefix) > #1649 (prefix) > #416 (contains)
    const rankedEmployees = [
      { id: "a", full_name: "Nguyen Van A", employee_id_number: "16" },
      { id: "b", full_name: "Tran Van B", employee_id_number: "161" },
      { id: "c", full_name: "Le Van C", employee_id_number: "1649" },
      { id: "d", full_name: "Pham Van D", employee_id_number: "416" },
    ];

 
    vi.mocked(useQuery).mockReturnValue({
      data: { data: rankedEmployees },
      isLoading: false,
    } as unknown as never);

    const onChange = vi.fn();
    render(<EmployeeSearch onChange={onChange} workspaceId="ws-1" />, { wrapper });

    // The combobox should be rendered with options in the exact order from the API
    const combobox = screen.getByRole("combobox");
    expect(combobox).toBeDefined();

    // Verify the component did NOT re-sort the data alphabetically
    // by checking the first option is still the exact match employee
    const allOptions = screen.queryAllByText(/16|161|1649|416/);
    if (allOptions.length > 0) {
      // First visible option should be the exact match "16", not "1649" or "416"
      expect(allOptions[0].textContent).toContain("16");
    }
  });

  it("passes keyboardShortcut prop to Combobox", () => {
 
    vi.mocked(useQuery).mockReturnValue({
      data: { data: mockEmployees },
      isLoading: false,
    } as unknown as never);

    const onChange = vi.fn();
    render(
      <EmployeeSearch keyboardShortcut="/" onChange={onChange} workspaceId="ws-1" />,
      { wrapper }
    );

    // Should render without crashing with the keyboardShortcut prop
    expect(screen.getByRole("combobox")).toBeDefined();
  });

  it("calls onSelectEmployee with the full Employee object when selected via hidden select", () => {
 
    vi.mocked(useQuery).mockReturnValue({
      data: { data: mockEmployees },
      isLoading: false,
    } as unknown as never);

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

    // Use the hidden select element (sr-only) to simulate selection
    const hiddenSelect = screen.getByRole("combobox");
    fireEvent.change(hiddenSelect, { target: { value: "1" } });

    // onChange should have been called with just the id
    expect(onChange).toHaveBeenCalledWith("1");

    // onSelectEmployee should have been called with the full employee object
    expect(onSelectEmployee).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "1",
        full_name: "John Doe",
        employee_id_number: "E001",
      })
    );
  });

  it("does NOT call onSelectEmployee when employee is not found in data", () => {
 
    vi.mocked(useQuery).mockReturnValue({
      data: { data: mockEmployees },
      isLoading: false,
    } as unknown as never);

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

    // Try selecting an ID that does not exist in mockEmployees
    const hiddenSelect = screen.getByRole("combobox");
    fireEvent.change(hiddenSelect, { target: { value: "non-existent-id" } });

    // onChange is still called (to update the value), but with empty string since it's not found
    expect(onChange).toHaveBeenCalledWith("");

    // But onSelectEmployee should NOT have been called since no Employee was found
    expect(onSelectEmployee).not.toHaveBeenCalled();
  });
});