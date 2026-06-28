import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import ResetPasswordPage from "../reset-password-page";

// Mock TanStack Router — providing a token via useSearch
const mockUseSearch = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useSearch: (opts?: { from?: string }) => mockUseSearch(opts),
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
    t: (key: string) => {
      const translations: Record<string, string> = {
        "auth.slogan": "HR Management System 4.0",
        "auth.reset_password_title": "Create new password",
        "auth.back_to_login": "Back to Login",
        "auth.login.title": "Login",
        "auth.new_password": "New Password",
        "auth.reset_password_btn": "Set New Password",
        "auth.reset_success":
          "Password changed! Please log in with your new password.",
        "auth.reset_token_invalid":
          "This reset link is invalid or has expired.",
        "auth.password_too_short":
          "Password must be at least 6 characters",
        "auth.forgot_password": "Forgot password?",
        "common.required_fields": "Please fill in all required fields",
        "common.processing": "Processing...",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock APIs
vi.mock("../../lib/api", () => ({
  resetPassword: vi.fn(),
}));

describe("ResetPasswordPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default: provide a valid token
    mockUseSearch.mockReturnValue({ token: "valid-reset-token-abc" });
  });

  const renderPage = () => render(<ResetPasswordPage />);

  describe("with a valid token", () => {
    it("renders the reset password form", () => {
      renderPage();

      expect(screen.getByText("Create new password")).toBeInTheDocument();
      expect(screen.getByLabelText("New Password")).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: "Set New Password" }),
      ).toBeInTheDocument();
    });

    it("shows validation error when password is empty", async () => {
      const user = userEvent.setup();
      renderPage();

      await user.click(
        screen.getByRole("button", { name: "Set New Password" }),
      );

      expect(
        screen.getByText("Please fill in all required fields"),
      ).toBeInTheDocument();
      expect(api.resetPassword).not.toHaveBeenCalled();
    });

    it("shows validation error when password is too short", async () => {
      const user = userEvent.setup();
      renderPage();

      await user.type(screen.getByLabelText("New Password"), "12345");
      await user.click(
        screen.getByRole("button", { name: "Set New Password" }),
      );

      expect(
        screen.getByText("Password must be at least 6 characters"),
      ).toBeInTheDocument();
      expect(api.resetPassword).not.toHaveBeenCalled();
    });

    it("calls resetPassword API and shows success state", async () => {
      const user = userEvent.setup();
      vi.mocked(api.resetPassword).mockResolvedValue(
        "Password reset successfully",
      );

      renderPage();

      await user.type(
        screen.getByLabelText("New Password"),
        "newSecurePass1",
      );
      await user.click(
        screen.getByRole("button", { name: "Set New Password" }),
      );

      await waitFor(() => {
        expect(api.resetPassword).toHaveBeenCalledWith(
          "valid-reset-token-abc",
          "newSecurePass1",
        );
      });

      // Should show success state
      expect(
        screen.getByText(
          "Password changed! Please log in with your new password.",
        ),
      ).toBeInTheDocument();

      // Should have login links (subtitle + success state both have one)
      const loginLinks = screen.getAllByRole("link", { name: "Login" });
      expect(loginLinks.some((l) => l.getAttribute("href") === "/login")).toBe(true);
    });

    it("displays error message on API failure", async () => {
      const user = userEvent.setup();
      vi.mocked(api.resetPassword).mockRejectedValue(
        new Error("This reset link is invalid or has expired."),
      );

      renderPage();

      await user.type(
        screen.getByLabelText("New Password"),
        "newSecurePass1",
      );
      await user.click(
        screen.getByRole("button", { name: "Set New Password" }),
      );

      await waitFor(() => {
        expect(api.resetPassword).toHaveBeenCalled();
      });

      expect(
        screen.getByText("This reset link is invalid or has expired."),
      ).toBeInTheDocument();

      // Should still show the form
      expect(screen.getByLabelText("New Password")).toBeInTheDocument();
    });
  });

  describe("without a token", () => {
    beforeEach(() => {
      mockUseSearch.mockReturnValue({ token: undefined });
    });

    it("shows missing token state with warning", () => {
      renderPage();

      expect(
        screen.getByText("This reset link is invalid or has expired."),
      ).toBeInTheDocument();

      // Should NOT show the password form
      expect(
        screen.queryByLabelText("New Password"),
      ).not.toBeInTheDocument();

      // Should show a link to request a new reset link
      const forgotLink = screen.getByText(/Forgot password/);
      expect(forgotLink.closest("a")).toHaveAttribute(
        "href",
        "/forgot-password",
      );
    });
  });
});
