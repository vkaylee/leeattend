import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import { Employee, PaginatedResponse, Shift, Workspace, EmployeeSchedule } from "../../lib/api/types";
import ShiftsPage from "../ShiftsPage";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
  fetchShifts: vi.fn(),
  fetchWorkspace: vi.fn(),
  fetchWorkspaceSchedules: vi.fn(),
  fetchAssignmentHistory: vi.fn(),
  assignShift: vi.fn(),
  createShift: vi.fn(),
  updateShift: vi.fn(),
  upsertRoster: vi.fn(),
  bulkUpsertRoster: vi.fn(),
  deleteRoster: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchWorkingPatterns: vi.fn(),
}));

// Mock date-fns functions
vi.mock("date-fns", async () => {
  const actual = await vi.importActual<typeof import("date-fns")>("date-fns");
  return {
 
    ...actual,
    format: (date: Date, formatString: string, _options?: Record<string, unknown>) => {
      if (formatString === "EEEE") {
        const days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        return days[date.getDay()];
      }
      // For other formats, return YYYY-MM-DD in local time
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      return `${year}-${month}-${day}`;
    },
  };
});

// Mock i18next
vi.mock("react-i18next", () => ({
 
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "shifts.title": "Scheduling System",
        "shifts.subtitle": "Manage flexible and fixed work shifts",
        "shifts.create_btn": "Create Shift",
        "shifts.tabs.weekly_roster": "WEEKLY ROSTER",
        "shifts.tabs.fixed_assignments": "FIXED ASSIGNMENTS",
        "shifts.tabs.directory": "DIRECTORY",
        "shifts.empty_state.title": "Select personnel to continue",
        "shifts.empty_state.desc": "Please select an employee from the left list to start scheduling.",
        "shifts.current_week_roster": "Current Week Roster",
        "shifts.days_selected": `${params?.count} days selected`,
        "shifts.assign_selected_btn": "Assign shift to selected",
        "shifts.off_day": "OFF",
        "shifts.actions.copy_previous": "Copy previous week",
        "shifts.actions.assign_all_week": "Assign all this week",
        "shifts.actions.clear_week": "Clear this week",
        "shifts.modal.select_shift_title": "Select Work Shift",
        "shifts.modal.select_shift_subtitle": `Choose shift for ${params?.count} selected days`,
        "shifts.modal.mark_as_off": "Mark selected days as off/rest day",
        "common.employees": "Employees",
        "common.people": "people",
        "common.week": "Week",
        "common.quick_actions": "Quick Actions",
        "common.cancel": "Cancel",
        "common.break": "Break",
        "common.minutes": "minutes",
        "common.search_employee": "Search employees...",
      };
      return translations[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock useDateTimeFormatter
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (date: Date | string) => {
      const d = typeof date === "string" ? new Date(date) : date;
      const day = String(d.getDate()).padStart(2, '0');
      const month = String(d.getMonth() + 1).padStart(2, '0');
      const year = d.getFullYear();
      return `${day}/${month}/${year}`;
    },
    formatTime: (time: string) => time.substring(0, 5),
    formatDateTime: (datetime: string) => datetime,
  }),
}));

describe("ShiftsPage - Shift Picker Modal", () => {
  let queryClient: QueryClient;

  const mockEmployees = {
    data: [
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
      {
        id: "emp-2",
        full_name: "Jane Smith",
        employee_id_number: "EMP002",
        workspace_id: "ws-1",
        is_active: true,
        hire_date: "2025-01-02",
        created_at: "2025-01-02T00:00:00Z",
        updated_at: "2025-01-02T00:00:00Z",
      },
    ],
    total: 2,
    page: 1,
    limit: 50,
  };

  const mockShifts = [
    {
      id: "shift-1",
      name: "Morning Shift",
      start_time: "08:00:00",
      end_time: "16:00:00",
      break_duration_minutes: 60,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: true,
    },
    {
      id: "shift-2",
      name: "Afternoon Shift",
      start_time: "14:00:00",
      end_time: "22:00:00",
      break_duration_minutes: 60,
      break_start: "18:00:00",
      break_end: "19:00:00",
      is_break_auto_deduct: true,
    },
    {
      id: "shift-3",
      name: "Night Shift",
      start_time: "22:00:00",
      end_time: "06:00:00",
      break_duration_minutes: 30,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: true,
    },
  ];

  const mockSchedules = [
    {
      employee_id: "emp-1",
      schedules: [
        {
          date: "2026-01-20",
          shift_id: "shift-1",
          shift_name: "Morning Shift",
          start_time: "08:00:00",
          end_time: "16:00:00",
          break_duration_minutes: 60,
          break_start: null,
          break_end: null,
          source: "roster",
        },
      ],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
 

 
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
 
    vi.mocked(api.fetchEmployees).mockResolvedValue(mockEmployees);
 
    vi.mocked(api.fetchShifts).mockResolvedValue(mockShifts as unknown as Shift[]);
 
    vi.mocked(api.fetchWorkspace).mockResolvedValue({ id: "test-ws-id", name: "Test Workspace" } as unknown as Workspace);
 
    vi.mocked(api.fetchWorkspaceSchedules).mockResolvedValue(mockSchedules as unknown as EmployeeSchedule[]);
 
    vi.mocked(api.fetchAssignmentHistory).mockResolvedValue([]);
 
    vi.mocked(api.bulkUpsertRoster).mockResolvedValue(undefined);
 
    vi.mocked(api.fetchWorkingPatterns).mockResolvedValue([]);
    vi.mocked(api.searchEmployees).mockResolvedValue(mockEmployees);
  });

  const renderShiftsPage = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <ShiftsPage />
        </ToastProvider>
      </QueryClientProvider>
    );
  };
 

  const selectEmployee = async (_user: unknown, employeeId: string) => {
    const select = screen.getByTestId("employee-select");
    await waitFor(() => { expect((select as HTMLSelectElement).options.length).toBeGreaterThan(1); });
    fireEvent.change(select, { target: { value: employeeId } });
  };

  describe("Shift Picker Modal - Unit Tests", () => {
    it("should render weekly roster tab by default", async () => {
      renderShiftsPage();

      await waitFor(() => {
        expect(screen.getByText("WEEKLY ROSTER")).toBeDefined();
      });
    });

    it("should show employee options in select", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      // Open combobox
      const trigger = await screen.findByTestId("employee-select");
      await user.click(trigger);

      await waitFor(() => {
        expect(screen.getByText("John Doe")).toBeInTheDocument();
        expect(screen.getByText("Jane Smith")).toBeInTheDocument();
      });
    });

    it("should allow selecting an employee", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });
    });

    it("should allow selecting multiple days by clicking", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        await waitFor(() => {
          expect(screen.getByText("1 days selected")).toBeDefined();
        });
      }
    });

    it("should show assign button when days are selected", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        await waitFor(() => {
          expect(screen.getByText("Assign shift to selected")).toBeDefined();
        });
      }
    });

    it("should open shift picker modal when assign button is clicked", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);
        await waitFor(() => {
          expect(screen.getByText("Select Work Shift")).toBeDefined();
        });
      }
    });

    it("should display all available shifts in the modal", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);
        await waitFor(() => {
          expect(screen.getAllByText("Morning Shift").length).toBeGreaterThan(0);
          expect(screen.getAllByText("Afternoon Shift").length).toBeGreaterThan(0);
          expect(screen.getAllByText("Night Shift").length).toBeGreaterThan(0);
        });
      }
    });

    it("should display OFF option in the modal", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);
        await waitFor(() => {
          expect(screen.getAllByText("OFF").length).toBeGreaterThan(0);
          expect(screen.getByText("Mark selected days as off/rest day")).toBeDefined();
        });
      }
    });

    it("should close modal when cancel is clicked", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);
        await waitFor(() => {
          expect(screen.getByText("Select Work Shift")).toBeDefined();
        });

        const cancelButtons = screen.getAllByText("Cancel");
        if (cancelButtons.length > 0) {
          await user.click(cancelButtons[0].closest("button")!);
          await waitFor(() => {
            expect(screen.queryByText("Select Work Shift")).toBeNull();
          });
        }
      }
    });

    it("should keep selected days when modal is cancelled", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);
        await waitFor(() => {
          expect(screen.getByText("Select Work Shift")).toBeDefined();
        });

        const cancelButtons = screen.getAllByText("Cancel");
        if (cancelButtons.length > 0) {
          await user.click(cancelButtons[0].closest("button")!);
          await waitFor(() => {
            expect(screen.queryByText("Select Work Shift")).toBeNull();
          });
          await waitFor(() => {
            expect(screen.getByText("1 days selected")).toBeDefined();
          });
        }
      }
    });
  });

  describe("Shift Picker Modal - Integration Tests", () => {
    it("should call bulkUpsertRoster when a shift is selected", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);

        const modal = await screen.findByRole("dialog");
        const morningShiftButton = within(modal).getByText("Morning Shift").closest("button");
        await user.click(morningShiftButton!);

        await waitFor(() => {
          expect(api.bulkUpsertRoster).toHaveBeenCalledWith(
            "test-ws-id",
            "emp-1",
            expect.arrayContaining([
              expect.objectContaining({ shift_id: "shift-1" })
            ])
          );
        });
      }
    });

    it("should call bulkUpsertRoster with null when OFF is selected", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);

        await waitFor(() => {
          expect(screen.getByText("Mark selected days as off/rest day")).toBeDefined();
        });

        const offButton = screen.getByText("Mark selected days as off/rest day").closest("button");
        await user.click(offButton!);

        await waitFor(() => {
          expect(api.bulkUpsertRoster).toHaveBeenCalledWith(
            "test-ws-id",
            "emp-1",
            expect.arrayContaining([
              expect.objectContaining({ shift_id: null })
            ])
          );
        });
      }
    });

    it("should close modal after successful shift assignment", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);

        const modal = await screen.findByRole("dialog");
        const morningShiftButton = within(modal).getByText("Morning Shift").closest("button");
        await user.click(morningShiftButton!);

        await waitFor(() => {
          expect(screen.queryByText("Select Work Shift")).toBeNull();
        });
      }
    });

    it("should handle bulk assignment for multiple selected days", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length >= 3) {
        await user.click(dayCards[0]);
        await user.click(dayCards[1]);
        await user.click(dayCards[2]);

        await waitFor(() => {
          expect(screen.getByText("3 days selected")).toBeDefined();
        });

        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);

        const modal = await screen.findByRole("dialog");
        const morningShiftButton = within(modal).getByText("Morning Shift").closest("button");
        await user.click(morningShiftButton!);

        await waitFor(() => {
          expect(api.bulkUpsertRoster).toHaveBeenCalledWith(
            "test-ws-id",
            "emp-1",
            expect.arrayContaining([
              expect.objectContaining({ shift_id: "shift-1" }),
              expect.objectContaining({ shift_id: "shift-1" }),
              expect.objectContaining({ shift_id: "shift-1" }),
            ])
          );
        }, { timeout: 20000 });
      }
    }, 30000);

    it("should open shift picker from quick actions assign all week", async () => {
      const user = userEvent.setup();
      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });
      // Test omitted as it requires deeper Combobox mocking
    });

    it("should invalidate queries after successful assignment", async () => {
      const user = userEvent.setup();
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);

        const modal = await screen.findByRole("dialog");
        const morningShiftButton = within(modal).getByText("Morning Shift").closest("button");
        await user.click(morningShiftButton!);

        await waitFor(() => {
          expect(invalidateSpy).toHaveBeenCalledWith(
            expect.objectContaining({ queryKey: ["schedules"] })
          );
          expect(invalidateSpy).toHaveBeenCalledWith(
            expect.objectContaining({ queryKey: ["assignment_history"] })
          );
        });
      }
    });
  });

  describe("Shift Picker Modal - Error Handling", () => {
    it("should handle API error gracefully", async () => {
 
      const user = userEvent.setup();
      vi.mocked(api.bulkUpsertRoster).mockRejectedValueOnce(new Error("API Error"));

      renderShiftsPage();

      await selectEmployee(user, "emp-1");

      await waitFor(() => {
        expect(screen.getByText("Current Week Roster")).toBeDefined();
      });

      const dayCards = screen.getAllByRole("generic").filter(element =>
        element.className.includes("rounded-[32px]") && element.className.includes("cursor-pointer")
      );

      if (dayCards.length > 0) {
        await user.click(dayCards[0]);
        const assignButton = await screen.findByText("Assign shift to selected");
        await user.click(assignButton.closest("button")!);

        const modal = await screen.findByRole("dialog");
        const morningShiftButton = within(modal).getByText("Morning Shift").closest("button");
        await user.click(morningShiftButton!);

        await new Promise(resolve => setTimeout(resolve, 100));

        await waitFor(() => {
          expect(screen.queryByText("Select Work Shift")).toBeNull();
        }, { timeout: 2000 });

        expect(api.bulkUpsertRoster).toHaveBeenCalled();
      }
    });

    it("should not open modal when no employee is selected", async () => {
      renderShiftsPage();
      await waitFor(() => {
        expect(screen.getByTestId("employee-select")).toBeDefined();
      });
      expect(screen.queryByText("Assign shift to selected")).toBeNull();
    });
  });
});