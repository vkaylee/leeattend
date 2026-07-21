import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

// ── Hoisted mocks ──────────────────────────────────────────────────────

const { mockFetchSmtpConfig, mockSaveSmtpConfig, mockTestSmtp, mockInvalidate, mockToast } =
  vi.hoisted(() => ({
    mockFetchSmtpConfig: vi.fn().mockResolvedValue({}),
    mockSaveSmtpConfig: vi.fn().mockResolvedValue(undefined),
    mockTestSmtp: vi
      .fn()
      .mockResolvedValue({ success: false, message: "test" }),
    mockInvalidate: vi.fn(),
    mockToast: { success: vi.fn(), error: vi.fn() },
  }));

// Query data that can be mutated per test
let queryData: Record<string, unknown> | null = {};
let queryLoading = false;

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  Link: ({
    to,
    children,
    ...props
  }: {
    to: string;
    children: React.ReactNode;
  }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => {
      const translations: Record<string, string> = {
        "smtp.title": "SMTP Email Settings",
        "smtp.description": "Configure outgoing email",
        "smtp.config_section": "SMTP Configuration",
        "smtp.test_section": "Test Email Delivery",
        "smtp.test_desc": "Send a test email to verify",
        "smtp.test_email_label": "Send test to",
        "smtp.test_btn": "Test Now",
        "smtp.testing": "Sending test email...",
        "smtp.host": "SMTP Host",
        "smtp.port": "Port",
        "smtp.username": "Username",
        "smtp.password": "Password / App Password",
        "smtp.from": "From Address",
        "smtp.base_url": "App Base URL",
        "smtp.base_url_hint": "Used in password reset links",
        "smtp.verified_badge": "Verified",
        "smtp.untested_badge": "Untested — send a test email to verify",
        "smtp.untested_reminder": "You haven't tested this configuration yet.",
        "common.save": "Save",
        "common.saved_success": "Saved successfully",
        "common.processing": "Processing...",
      };
      return translations[key] || fallback || key;
    },
  }),
}));

vi.mock("@tanstack/react-query", () => ({
  useQuery: () => ({ data: queryData, isLoading: queryLoading }),
  useMutation: ({
    mutationFn,
    onSuccess,
    onError,
  }: {
    mutationFn: (data: unknown) => Promise<unknown>;
    onSuccess?: () => void;
    onError?: (err: Error) => void;
  }) => ({
    mutate: (data: unknown) => {
      mutationFn(data)
        .then(() => onSuccess?.())
        .catch((err: Error) => onError?.(err));
    },
    isPending: false,
  }),
  useQueryClient: () => ({ invalidateQueries: mockInvalidate }),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => mockToast,
}));

vi.mock("@/lib/api/smtp-config", () => ({
  fetchSmtpConfig: mockFetchSmtpConfig,
  saveSmtpConfig: mockSaveSmtpConfig,
  testSmtp: mockTestSmtp,
}));

// Must come after all vi.mock calls
import SmtpSettingsPage from "../smtp-settings-page";

describe("SmtpSettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    queryData = {};
    queryLoading = false;
    mockFetchSmtpConfig.mockResolvedValue({});
  });

  const renderPage = () => render(<SmtpSettingsPage />);

  it("renders the SMTP settings form with all fields", () => {
    queryData = {
      host: "smtp.gmail.com",
      port: 587,
      username: "admin@test.com",
      password: "",
      from: "",
      base_url: "",
    };

    renderPage();

    expect(screen.getByTestId("smtp-field-host")).toBeDefined();
    expect(screen.getByTestId("smtp-field-port")).toBeDefined();
    expect(screen.getByTestId("smtp-field-username")).toBeDefined();
    expect(screen.getByTestId("smtp-field-password")).toBeDefined();
    expect(screen.getByTestId("smtp-save-btn")).toBeDefined();
    expect(screen.getByTestId("smtp-test-email-input")).toBeDefined();
    expect(screen.getByTestId("smtp-test-btn")).toBeDefined();
  });

  it("shows skeleton while loading", () => {
    queryLoading = true;
    queryData = null;

    renderPage();

    // Should show skeletons, not the form fields
    expect(screen.queryByTestId("smtp-field-host")).toBeNull();
    expect(screen.queryByTestId("smtp-save-btn")).toBeNull();
  });

  it("enables save button when form is dirty", async () => {
    queryData = {};

    renderPage();

    const saveBtn = screen.getByTestId("smtp-save-btn");
    expect(saveBtn).toBeDisabled();

    const user = userEvent.setup();
    const hostInput = screen.getByTestId("smtp-field-host");
    await user.clear(hostInput);
    await user.type(hostInput, "smtp.gmail.com");

    expect(saveBtn).not.toBeDisabled();
  });

  it("calls saveSmtpConfig and shows success toast", async () => {
    mockSaveSmtpConfig.mockResolvedValue(undefined);
    queryData = {};

    renderPage();

    const user = userEvent.setup();
    const hostInput = screen.getByTestId("smtp-field-host");
    const portInput = screen.getByTestId("smtp-field-port") as HTMLInputElement;

    await user.clear(hostInput);
    await user.type(hostInput, "smtp.gmail.com");
    // Clear default 587 before typing to avoid "587587"
    await user.clear(portInput);
    await user.type(portInput, "587");

    await user.click(screen.getByTestId("smtp-save-btn"));

    await waitFor(() => {
      expect(mockSaveSmtpConfig).toHaveBeenCalledWith({
        host: "smtp.gmail.com",
        port: 587,
        username: undefined,
        password: undefined,
        from: undefined,
        base_url: undefined,
        http_api: null,
      });
      expect(mockToast.success).toHaveBeenCalled();
      expect(mockInvalidate).toHaveBeenCalledWith({ queryKey: ["smtp-config"] });
    });
  });

  it("shows error toast on save failure", async () => {
    mockSaveSmtpConfig.mockRejectedValue(new Error("DB error"));
    queryData = {};

    renderPage();

    const user = userEvent.setup();
    const hostInput = screen.getByTestId("smtp-field-host");
    await user.clear(hostInput);
    await user.type(hostInput, "smtp.test.com");
    await user.click(screen.getByTestId("smtp-save-btn"));

    await waitFor(() => {
      expect(mockToast.error).toHaveBeenCalledWith("DB error");
    });
  });

  it("calls testSmtp and displays success result", async () => {
    mockTestSmtp.mockResolvedValue({
      success: true,
      message: "Test email sent successfully.",
    });
    queryData = {};

    renderPage();

    const user = userEvent.setup();
    await user.type(
      screen.getByTestId("smtp-test-email-input"),
      "admin@company.com",
    );
    await user.click(screen.getByTestId("smtp-test-btn"));

    await waitFor(() => {
      expect(mockTestSmtp).toHaveBeenCalledWith("admin@company.com");
    });

    const result = screen.getByTestId("smtp-test-result");
    expect(result.textContent).toContain("Test email sent successfully.");
  });

  it("calls testSmtp and displays failure result", async () => {
    mockTestSmtp.mockResolvedValue({
      success: false,
      message: "SMTP is not configured.",
    });
    queryData = {};

    renderPage();

    const user = userEvent.setup();
    await user.type(
      screen.getByTestId("smtp-test-email-input"),
      "admin@company.com",
    );
    await user.click(screen.getByTestId("smtp-test-btn"));

    await waitFor(() => {
      const result = screen.getByTestId("smtp-test-result");
      expect(result.textContent).toContain("SMTP is not configured.");
    });
  });

  it("disables test button when email is empty", () => {
    queryData = {};
    renderPage();
    expect(screen.getByTestId("smtp-test-btn")).toBeDisabled();
  });

  it("shows untested badge initially", () => {
    queryData = {};
    renderPage();
    expect(screen.getByText(/Untested/)).toBeDefined();
  });

  it("shows untested reminder when not verified", () => {
    queryData = {};
    renderPage();
    expect(
      screen.getByText(/You haven't tested this configuration yet/),
    ).toBeDefined();
  });

  it("shows verified badge after successful test", async () => {
    mockTestSmtp.mockResolvedValue({
      success: true,
      message: "OK",
    });
    queryData = {};

    renderPage();

    const user = userEvent.setup();
    await user.clear(screen.getByTestId("smtp-test-email-input"));
    await user.type(
      screen.getByTestId("smtp-test-email-input"),
      "admin@company.com",
    );
    await user.click(screen.getByTestId("smtp-test-btn"));

    await waitFor(() => {
      expect(screen.getByText(/Verified/)).toBeDefined();
    });
  });

  it("resets to untested after save", async () => {
    mockTestSmtp.mockResolvedValue({
      success: true,
      message: "OK",
    });
    mockSaveSmtpConfig.mockResolvedValue(undefined);
    queryData = {};

    renderPage();

    const user = userEvent.setup();
    // First test to get verified
    await user.clear(screen.getByTestId("smtp-test-email-input"));
    await user.type(screen.getByTestId("smtp-test-email-input"), "admin@company.com");
    await user.click(screen.getByTestId("smtp-test-btn"));

    await waitFor(() => {
      expect(screen.getByText(/Verified/)).toBeDefined();
    });

    // Then save → should reset to untested
    await user.clear(screen.getByTestId("smtp-field-host"));
    await user.type(screen.getByTestId("smtp-field-host"), "smtp.new.com");
    await user.click(screen.getByTestId("smtp-save-btn"));

    await waitFor(() => {
      expect(screen.getByText(/Untested/)).toBeDefined();
    });
  });
});
