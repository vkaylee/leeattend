import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import { ROUTES } from "../../lib/routes";
import RegisterPage from "../register-page";

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
        "auth.register_title": "Register",
        "auth.slogan": "Your Company Slogan",
        "auth.password_mismatch": "Passwords do not match",
        "auth.password_too_short": "Password too short",
        "auth.registration_failed": "Registration failed",
        "auth.display_name": "Full Name",
        "auth.email": "Email",
        "auth.password": "Password",
        "auth.confirm_password": "Confirm Password",
        "auth.creating_account": "Creating Account...",
        "auth.register_btn": "Register Now",
        "auth.already_have_account": "Already have an account?",
        "auth.login.title": "Login",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock APIs
vi.mock("../../lib/api", () => ({
  register: vi.fn(),
  setAccessToken: vi.fn(),
  setRefreshToken: vi.fn(),
  setCurrentUserId: vi.fn(),
  getCurrentUserId: vi.fn(),
}));

describe("RegisterPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.getCurrentUserId).mockReturnValue(null);
    // Mock window.location.href
    Object.defineProperty(window, "location", {
      value: { href: "" },
      writable: true,
    });
  });

  const renderPage = () => render(<RegisterPage />);

  it("renders registration form", () => {
    renderPage();
    expect(screen.getByText("Register")).toBeInTheDocument();
    expect(screen.getByLabelText("Full Name")).toBeInTheDocument();
    expect(screen.getByLabelText("Email")).toBeInTheDocument();
    expect(screen.getByLabelText("Password")).toBeInTheDocument();
    expect(screen.getByLabelText("Confirm Password")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "REGISTER NOW" })).toBeInTheDocument();
  });

  it("redirects to HOME if user is already logged in", () => {
    vi.mocked(api.getCurrentUserId).mockReturnValue("user-1");
    renderPage();
    expect(mockNavigate).toHaveBeenCalledWith({ to: ROUTES.HOME, replace: true });
  });

  it("shows error if passwords do not match", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.type(screen.getByLabelText("Full Name"), "Test");
    await user.type(screen.getByLabelText("Email"), "test@test.com");
    await user.type(screen.getByLabelText("Password"), "password123");
    await user.type(screen.getByLabelText("Confirm Password"), "password456");
    await user.click(screen.getByRole("button", { name: "REGISTER NOW" }));

    expect(screen.getByText("Passwords do not match")).toBeInTheDocument();
    expect(api.register).not.toHaveBeenCalled();
  });

  it("shows error if password is too short", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.type(screen.getByLabelText("Full Name"), "Test");
    await user.type(screen.getByLabelText("Email"), "test@test.com");
    await user.type(screen.getByLabelText("Password"), "12345");
    await user.type(screen.getByLabelText("Confirm Password"), "12345");
    await user.click(screen.getByRole("button", { name: "REGISTER NOW" }));

    expect(screen.getByText("Password too short")).toBeInTheDocument();
    expect(api.register).not.toHaveBeenCalled();
  });

  it("calls API and redirects on successful registration", async () => {
    const user = userEvent.setup();
    const mockResponse = {
      user: { id: "user-123" },
      access_token: "access-token",
      refresh_token: "refresh-token",
    };
    vi.mocked(api.register).mockResolvedValue(mockResponse as never);

    renderPage();

    await user.type(screen.getByLabelText("Full Name"), "Test User");
    await user.type(screen.getByLabelText("Email"), "test@example.com");
    await user.type(screen.getByLabelText("Password"), "password123");
    await user.type(screen.getByLabelText("Confirm Password"), "password123");

    await user.click(screen.getByRole("button", { name: "REGISTER NOW" }));

    await waitFor(() => {
      expect(api.register).toHaveBeenCalledWith({
        email: "test@example.com",
        password: "password123",
        full_name: "Test User",
      });
    });

    expect(api.setCurrentUserId).toHaveBeenCalledWith("user-123");
    expect(api.setAccessToken).toHaveBeenCalledWith("access-token");
    expect(api.setRefreshToken).toHaveBeenCalledWith("refresh-token");
    expect(window.location.href).toBe(ROUTES.ONBOARDING);
  });

  it("shows API error message on failure", async () => {
    const user = userEvent.setup();
    vi.mocked(api.register).mockRejectedValue(new Error("Email already taken"));

    renderPage();

    await user.type(screen.getByLabelText("Full Name"), "Test User");
    await user.type(screen.getByLabelText("Email"), "test@example.com");
    await user.type(screen.getByLabelText("Password"), "password123");
    await user.type(screen.getByLabelText("Confirm Password"), "password123");

    await user.click(screen.getByRole("button", { name: "REGISTER NOW" }));

    await waitFor(() => {
      expect(screen.getByText("Email already taken")).toBeInTheDocument();
    });
  });
});
