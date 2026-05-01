import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import * as useProfileHook from "../../hooks/use-profile";
import * as useThemeHook from "../../hooks/use-theme";
import * as api from "../../lib/api";
import ProfilePage from "../ProfilePage";

// Mock TanStack Router
const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "common.profile": "Profile",
        "profile.subtitle": "Manage your profile",
        "profile.personalization.title": "Personalization",
        "profile.security.title": "Security",
        "employees.table.name": "Full Name",
        "common.email": "Email",
        "profile.preferred_language": "Language",
        "profile.bio": "Bio",
        "common.save": "Save",
        "profile.security.current_password": "Current Password",
        "profile.security.new_password": "New Password",
        "profile.security.confirm_password": "Confirm Password",
        "profile.security.submit_btn": "Update Password",
        "profile.security.messages.confirm_mismatch": "Passwords mismatch",
        "profile.security.messages.password_changed": "Password changed",
        "common.colors.emerald": "Emerald",
        "common.colors.indigo": "Indigo",
        "common.colors.blue": "Blue",
        "common.colors.amber": "Amber",
        "common.colors.rose": "Rose",
        "common.colors.slate": "Slate",
        "common.colors.black": "Black",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock Combobox
vi.mock("../../components/ui/Combobox", () => ({
  Combobox: ({ options, value, onChange, placeholder, "data-testid": testId }: { options: { value: string; label: string }[]; value: string; onChange: (v: string) => void; placeholder?: string; "data-testid"?: string }) => (
    <select
      data-testid={testId || "mock-combobox"}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      <option value="">{placeholder || "Select..."}</option>
      {options.map((opt: { value: string; label: string }) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));

vi.mock("../../hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (date: string) => date,
  }),
}));

vi.mock("../../lib/api", () => ({
  changePassword: vi.fn(),
  clearAuthSession: vi.fn(),
}));

const mockUser = {
  id: "user-1",
  email: "test@example.com",
  full_name: "Test User",
  avatar_url: "",
  bio: "Developer",
  preferred_language: "en",
  created_at: "2026-01-01T00:00:00Z",
};

describe("ProfilePage", () => {
  let queryClient: QueryClient;
  const mockUpdateProfile = vi.fn();
  const mockSetUserTheme = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });

    vi.spyOn(useProfileHook, "useProfile").mockReturnValue({
      user: mockUser as ReturnType<typeof useProfileHook.useProfile>["user"],
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
      updateProfile: mockUpdateProfile,
      isUpdating: false,
    });

    vi.spyOn(useThemeHook, "useTheme").mockReturnValue({
      primaryColor: "#4f46e5",
      isOverridden: false,
      setUserTheme: mockSetUserTheme,
      setPreviewTheme: vi.fn(),
    });
  });

  const renderPage = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <ProfilePage />
        </ToastProvider>
      </QueryClientProvider>
    );

  it("renders profile tab by default", () => {
    renderPage();
    expect(screen.getAllByText("Profile").length).toBeGreaterThan(0);
    // User data is rendered
    expect(screen.getByText("test@example.com")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Test User")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Developer")).toBeInTheDocument();
  });

  it("updates profile on submit", async () => {
    const user = userEvent.setup();
    renderPage();

    const nameInput = screen.getByDisplayValue("Test User");
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Name");

    await user.click(screen.getByRole("button", { name: "SAVE" }));

    await waitFor(() => {
      expect(mockUpdateProfile).toHaveBeenCalledWith(
        expect.objectContaining({
          full_name: "Updated Name",
          bio: "Developer",
          preferred_language: "en",
        })
      );
    });
  });

  it("switches to personalization tab and changes theme", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(screen.getByText("Personalization"));
    
    // Check if colors are rendered (via titles)
    const emeraldBtns = screen.getAllByTitle("Emerald");
    expect(emeraldBtns.length).toBeGreaterThan(0);

    // Click a preset color
    await user.click(emeraldBtns[0]);
    expect(mockSetUserTheme).toHaveBeenCalledWith("#10b981");
  });

  it("switches to security tab and changes password", async () => {
    const user = userEvent.setup();
    vi.mocked(api.changePassword).mockResolvedValue(undefined as never);

    renderPage();
    await user.click(screen.getByText("Security"));

    const currentPw = screen.getByTestId("current-password-input");
    const newPw = screen.getByTestId("new-password-input");
    const confirmPw = screen.getByTestId("confirm-password-input");

    await user.type(currentPw, "old123");
    await user.type(newPw, "new123");
    await user.type(confirmPw, "new123");

    await user.click(screen.getByTestId("update-password-btn"));

    await waitFor(() => {
      expect(api.changePassword).toHaveBeenCalledWith({
        old_password: "old123",
        new_password: "new123",
      });
    });

    // We can't easily wait for the setTimeout here without vitest fake timers,
    // but the API call success is tested.
  });

  it("shows error if passwords do not match", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByText("Security"));

    const currentPw = screen.getByTestId("current-password-input");
    const newPw = screen.getByTestId("new-password-input");
    const confirmPw = screen.getByTestId("confirm-password-input");

    await user.type(currentPw, "old123");
    await user.type(newPw, "new123");
    await user.type(confirmPw, "different");

    await user.click(screen.getByTestId("update-password-btn"));

    // Ensure API is not called
    expect(api.changePassword).not.toHaveBeenCalled();
    // Toast warning is triggered
    expect(await screen.findByText("Passwords mismatch")).toBeInTheDocument();
  });
});
