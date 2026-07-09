import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import i18n from "../../i18n/config";
import * as api from "../../lib/api";
import LoginPage from "../login-page";

const { mockNavigate } = vi.hoisted(() => {
  return { mockNavigate: vi.fn() };
});

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  useLocation: () => ({ pathname: "/", search: "", hash: "", state: null }),
 
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock the API library
// NOTE: setRefreshToken is intentionally absent — the refresh token is no
// longer handled by the frontend (persisted as a HttpOnly cookie by the
// backend). The login page must NOT call any setRefreshToken.
vi.mock("../../lib/api", () => ({
  login: vi.fn(),
  getCurrentUserId: vi.fn(),
  setCurrentUserId: vi.fn(),
  setAccessToken: vi.fn(),
}));

describe("LoginPage", () => {
  const originalDevelopment = import.meta.env.DEV;

  beforeEach(() => {
    vi.clearAllMocks();
    void i18n.changeLanguage("en");
 
    vi.mocked(api.getCurrentUserId).mockReturnValue(null);
  });

  afterEach(() => {
    import.meta.env.DEV = originalDevelopment;
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(ui);
  };

  it("renders login form and handles successful submission", async () => {
    renderWithProviders(<LoginPage />);

    const emailInput = screen.getByTestId("login-email-input");
    const passwordInput = screen.getByTestId("login-password-input");
    const submitButton = screen.getByTestId("login-submit-btn");

    expect(emailInput).toBeDefined();
    expect(passwordInput).toBeDefined();

    const user = userEvent.setup();
 
    vi.mocked(api.login).mockResolvedValue({ 
      user: { id: "user-1", email: "admin@leeattend.com", full_name: "Admin", created_at: "" },
      access_token: "token", 
      refresh_token: "refresh" 
    });

    await user.type(emailInput, "admin@leeattend.com");
    await user.type(passwordInput, "admin123");
    await user.click(submitButton);

    await waitFor(() => {
      expect(api.login).toHaveBeenCalledWith("admin@leeattend.com", "admin123");
      expect(mockNavigate).toHaveBeenCalledWith({ to: "/", replace: true });
    });
  });

  it("displays error toast on invalid credentials", async () => {
    renderWithProviders(<LoginPage />);

    const emailInput = screen.getByTestId("login-email-input");
    const passwordInput = screen.getByTestId("login-password-input");
    const submitButton = screen.getByTestId("login-submit-btn");

    const user = userEvent.setup();
 
    vi.mocked(api.login).mockRejectedValue(new Error("Invalid email or password"));

    await user.type(emailInput, "admin@leeattend.com");
    await user.type(passwordInput, "wrongpassword");
    await user.click(submitButton);

    await waitFor(() => {
      expect(api.login).toHaveBeenCalledWith("admin@leeattend.com", "wrongpassword");
    });
    
    expect(await screen.findByRole("alert")).toHaveTextContent("Invalid email or password");
    expect(mockNavigate).not.toHaveBeenCalled();
  });

  it("redirects to home if user is already logged in", () => {
    vi.mocked(api.getCurrentUserId).mockReturnValue("user-123");
    
    renderWithProviders(<LoginPage />);
    
    // The component returns null, so no form should be visible
    expect(screen.queryByTestId("login-email-input")).toBeNull();
  });
});
