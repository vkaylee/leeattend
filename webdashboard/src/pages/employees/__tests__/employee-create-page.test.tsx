import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import { ToastProvider } from "@/components/ui/toast";

import EmployeeCreatePage from "../employee-create-page";

const navigate = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => navigate,
  useSearch: () => ({ enrollment_id: undefined }),
  useBlocker: () => ({ status: "unblocked" }),
  Link: ({ children, ...props }: { children: React.ReactNode; [key: string]: unknown }) => <a {...props}>{children}</a>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue ?? key,
  }),
}));

vi.mock("@/lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/api")>()),
  createEmployee: vi.fn(),
  fetchEmployers: vi.fn().mockResolvedValue([]),
  fetchLocations: vi.fn().mockResolvedValue([]),
}));

vi.mock("@/pages/settings/hooks/use-locations", () => ({
  useLocations: () => ({ data: [], isError: false }),
}));

function renderPage() {
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <ToastProvider><EmployeeCreatePage /></ToastProvider>
    </QueryClientProvider>,
  );
}

describe("EmployeeCreatePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders HR fields without account linking controls", () => {
    renderPage();

    expect(screen.getByTestId("employees-form-name-input")).toBeDefined();
    expect(screen.getByTestId("employees-form-id-input")).toBeDefined();
    expect(screen.getByTestId("employees-form-hire-date-input")).toBeDefined();
    expect(screen.queryByTestId("employees-form-user-combobox")).toBeNull();
  });

  it("creates employee without sending user_id", async () => {
    const user = userEvent.setup();
    vi.mocked(api.createEmployee).mockResolvedValue({ id: "employee-1" } as never);
    renderPage();

    await user.type(screen.getByTestId("employees-form-name-input"), "New Employee");
    await user.type(screen.getByTestId("employees-form-id-input"), "EMP-100");
    fireEvent.change(screen.getByTestId("employees-form-hire-date-input"), { target: { value: "2026-11-19" } });
    fireEvent.submit(document.getElementById("employee-create-form")!);

    await waitFor(() => {
      expect(api.createEmployee).toHaveBeenCalledTimes(1);
      const [payload] = vi.mocked(api.createEmployee).mock.calls[0];
      expect(payload).not.toHaveProperty("user_id");
    });
  });
});
