import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, act, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import Button from "@/components/ui/button";

import { useConfirm } from "../use-confirm";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
 
    t: (key: string, options?: { defaultValue?: string }) => {
      if (options?.defaultValue) {
        return options.defaultValue;
      }
      return key;
    },
  }),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

const TestComponent = ({ onResult }: { onResult: (res: boolean) => void }) => {
  const { confirm, ConfirmationDialog } = useConfirm();

  const handleAction = async () => {
    const result = await confirm({
      title: "Test Hook",
      message: "Are you sure you want to test this hook?",
      confirmLabel: "Yes",
      cancelLabel: "No"
    });
    onResult(result);
  };

  return (
    <div>
      <Button onClick={handleAction} variant="primary">Trigger Confirm</Button>
      <ConfirmationDialog />
    </div>
  );
};

describe("useConfirm Hook", () => {
  it("resolves to true when confirm is clicked", async () => {
    const onResult = vi.fn();
    render(<TestComponent onResult={onResult} />, { wrapper: createWrapper() });

    // Initially dialog should not be in document
    expect(screen.queryByText("Test Hook")).not.toBeInTheDocument();

    // Trigger the confirm dialog
    act(() => {
      fireEvent.click(screen.getByText("Trigger Confirm"));
    });

    expect(screen.getByText("Test Hook")).toBeInTheDocument();

    // Click Yes
    act(() => {
      fireEvent.click(screen.getByText("Yes"));
    });

    await waitFor(() => { expect(onResult).toHaveBeenCalledWith(true); });
  });

  it("resolves to false when cancel is clicked", async () => {
    const onResult = vi.fn();
    render(<TestComponent onResult={onResult} />, { wrapper: createWrapper() });

    // Trigger the confirm dialog
    act(() => {
      fireEvent.click(screen.getByText("Trigger Confirm"));
    });

    // Click No
    act(() => {
      fireEvent.click(screen.getByText("No"));
    });

    await waitFor(() => { expect(onResult).toHaveBeenCalledWith(false); });
  });
});
