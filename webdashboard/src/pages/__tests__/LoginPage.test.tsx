import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import LoginPage from "../LoginPage";
import { BrowserRouter } from "react-router-dom";
import * as api from "../../lib/api";
import React from "react";
import i18n from "../../i18n/config";

// Mock the API library
vi.mock("../../lib/api", () => ({
  login: vi.fn(),
  getCurrentUserId: vi.fn(),
  setCurrentUserId: vi.fn(),
  setAccessToken: vi.fn(),
  setRefreshToken: vi.fn(),
}));

describe("LoginPage", () => {
  const originalDev = import.meta.env.DEV;

  beforeEach(() => {
    vi.clearAllMocks();
    i18n.changeLanguage("en");
    (api.getCurrentUserId as any).mockReturnValue(null);
  });

  afterEach(() => {
    // Restore the original DEV value

    import.meta.env.DEV = originalDev;
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <BrowserRouter>
        {ui}
      </BrowserRouter>
    );
  };

  it("renders login form and fast login button in DEV mode", async () => {

    import.meta.env.DEV = true;
    renderWithProviders(<LoginPage />);

    expect(screen.getByTestId("login-email-input")).toBeDefined();
    expect(screen.getByTestId("login-password-input")).toBeDefined();
    
    // Fast login button should exist
    const fastLoginBtn = screen.getByTestId("fast-login-btn");
    expect(fastLoginBtn).toBeDefined();

    const user = userEvent.setup();
    (api.login as any).mockResolvedValue({});

    await user.click(fastLoginBtn);

    await waitFor(() => {
      expect(api.login).toHaveBeenCalledWith("admin@leeattend.com", "admin123");
    });
  });

  it("does not render fast login button in PROD mode", async () => {

    import.meta.env.DEV = false;
    renderWithProviders(<LoginPage />);

    // Fast login button should NOT exist
    expect(screen.queryByTestId("fast-login-btn")).toBeNull();
  });
});
