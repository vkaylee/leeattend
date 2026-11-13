import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as api from "@/lib/api";

import PayrollSettings from "../payroll-settings";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...mod,
  };
});

// Mock child components to isolate tests
vi.mock("@/components/payroll/bonus-policy-manager", () => ({ default: () => <div data-testid="bonus-policy-manager" /> }));
vi.mock("@/components/payroll/overtime-policy-manager", () => ({ default: () => <div data-testid="overtime-policy-manager" /> }));
vi.mock("@/components/payroll/payroll-rules-manager", () => ({ default: () => <div data-testid="payroll-rules-manager" /> }));
vi.mock("@/components/payroll/payroll-settings-section", () => ({ default: () => <div data-testid="payroll-settings-section" /> }));
vi.mock("@/components/payroll/salary-component-manager", () => ({ default: () => <div data-testid="salary-component-manager" /> }));

describe("PayrollSettings Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });


  it("renders tabs and default policy section", () => {
    render(<PayrollSettings />);

    expect(screen.getByTestId("payroll-tab-policy")).toBeDefined();
    expect(screen.getByTestId("payroll-settings-section")).toBeDefined();
  });

  it("switches to components tab", async () => {
    render(<PayrollSettings />);

    fireEvent.click(screen.getByTestId("payroll-group-formulas"));
    fireEvent.click(screen.getByTestId("payroll-tab-components"));
    await waitFor(() => {
      expect(screen.getByTestId("salary-component-manager")).toBeDefined();
    });
  });

  it("keeps components without the retired salary bases tab", () => {
    render(<PayrollSettings />);

    fireEvent.click(screen.getByTestId("payroll-group-formulas"));
    expect(screen.getByTestId("salary-component-manager")).toBeInTheDocument();
    expect(screen.queryByTestId("payroll-tab-bases")).not.toBeInTheDocument();
  });
  
  it("switches to incentives tab", async () => {
    render(<PayrollSettings />);

    fireEvent.click(screen.getByTestId("payroll-group-benefits"));
    fireEvent.click(screen.getByTestId("payroll-tab-incentives"));
    await waitFor(() => {
      expect(screen.getByTestId("bonus-policy-manager")).toBeDefined();
    });
  });

  it("switches to overtime tab", async () => {
    render(<PayrollSettings />);

    fireEvent.click(screen.getByTestId("payroll-group-benefits"));
    fireEvent.click(screen.getByTestId("payroll-tab-overtime"));
    await waitFor(() => {
      expect(screen.getByTestId("overtime-policy-manager")).toBeDefined();
    });
  });

  it("switches to statutory_rules tab", async () => {
    render(<PayrollSettings />);

    fireEvent.click(screen.getByTestId("payroll-group-core"));
    fireEvent.click(screen.getByTestId("payroll-tab-rules"));
    await waitFor(() => {
      expect(screen.getByTestId("payroll-rules-manager")).toBeDefined();
    });
  });
});
