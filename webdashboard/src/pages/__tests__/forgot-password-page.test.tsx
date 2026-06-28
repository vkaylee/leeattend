import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import ForgotPasswordPage from "../forgot-password-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useSearch: () => ({}),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "en", changeLanguage: () => Promise.resolve() },
    t: (key: string, vars?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "auth.slogan": "HR Management System 4.0",
        "auth.forgot_password_title": "Reset your password",
        "auth.forgot_password_desc":
          "Enter your email and we'll send you a password reset link.",
        "auth.reset_link_sent": "Check your email",
        "auth.reset_link_sent_desc":
          "If {{email}} is registered, you'll receive a link within a few minutes.",
        "auth.email": "Email",
        "auth.send_reset_link": "Send Reset Link",
        "auth.back_to_login": "Back to Login",
        "auth.login_error": "Incorrect email or password",
        "common.required_fields": "Please fill in all required fields",
        "common.processing": "Processing...",
      };
      let result = translations[key] || key;
      // Handle interpolation
      if (vars && typeof result === "string") {
        result = result.replace(/\{\{(\w+)\}\}/g, (_, name) =>
          String(vars[name] ?? `{{${name}}}`),
        );
      }
      return result;
    },
  }),
}));

// Mock APIs
vi.mock("../../lib/api", () => ({
  forgotPassword: vi.fn(),
}));

describe("ForgotPasswordPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const renderPage = () => render(<ForgotPasswordPage />);

  it("renders the forgot password form", () => {
    renderPage();

    expect(screen.getByText("Reset your password")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Enter your email and we'll send you a password reset link.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Email")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Send Reset Link" }),
    ).toBeInTheDocument();
  });

  it("shows validation error when email is empty", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(screen.getByRole("button", { name: "Send Reset Link" }));

    expect(
      screen.getByText("Please fill in all required fields"),
    ).toBeInTheDocument();
    expect(api.forgotPassword).not.toHaveBeenCalled();
  });

  it("calls forgotPassword API and shows success state", async () => {
    const user = userEvent.setup();
    vi.mocked(api.forgotPassword).mockResolvedValue(undefined);

    renderPage();

    await user.type(
      screen.getByLabelText("Email"),
      "user@example.com",
    );
    await user.click(screen.getByRole("button", { name: "Send Reset Link" }));

    await waitFor(() => {
      expect(api.forgotPassword).toHaveBeenCalledWith("user@example.com");
    });

    // Should show success state
    expect(screen.getByText("Check your email")).toBeInTheDocument();
    expect(
      screen.getByText(
        "If user@example.com is registered, you'll receive a link within a few minutes.",
      ),
    ).toBeInTheDocument();

    // Should have a back to login link
    const backLink = screen.getByText(/Back to Login/);
    expect(backLink.closest("a")).toHaveAttribute("href", "/login");
  });

  it("displays error message on API failure", async () => {
    const user = userEvent.setup();
    vi.mocked(api.forgotPassword).mockRejectedValue(
      new Error("SMTP not configured"),
    );

    renderPage();

    await user.type(
      screen.getByLabelText("Email"),
      "user@example.com",
    );
    await user.click(screen.getByRole("button", { name: "Send Reset Link" }));

    await waitFor(() => {
      expect(api.forgotPassword).toHaveBeenCalledWith("user@example.com");
    });

    expect(screen.getByText("SMTP not configured")).toBeInTheDocument();
    // Should still be on the form (not success state)
    expect(screen.getByLabelText("Email")).toBeInTheDocument();
  });

  it("shows back to login link on the form", () => {
    renderPage();

    const backLink = screen.getByText(/Back to Login/);
    expect(backLink.closest("a")).toHaveAttribute("href", "/login");
  });
});
