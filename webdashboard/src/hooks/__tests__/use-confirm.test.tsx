import { render, screen, fireEvent, act } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { useConfirm } from "../use-confirm";
import React from "react";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
      if (options && options.defaultValue) {
        return options.defaultValue;
      }
      return key;
    },
  }),
}));

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
      <button onClick={handleAction}>Trigger Confirm</button>
      <ConfirmationDialog />
    </div>
  );
};

describe("useConfirm Hook", () => {
  it("resolves to true when confirm is clicked", async () => {
    const onResult = vi.fn();
    render(<TestComponent onResult={onResult} />);

    // Initially dialog should not be in document
    expect(screen.queryByText("Test Hook")).not.toBeInTheDocument();

    // Trigger the confirm dialog
    await act(async () => {
      fireEvent.click(screen.getByText("Trigger Confirm"));
    });

    expect(screen.getByText("Test Hook")).toBeInTheDocument();

    // Click Yes
    await act(async () => {
      fireEvent.click(screen.getByText("Yes"));
    });

    expect(onResult).toHaveBeenCalledWith(true);
  });

  it("resolves to false when cancel is clicked", async () => {
    const onResult = vi.fn();
    render(<TestComponent onResult={onResult} />);

    // Trigger the confirm dialog
    await act(async () => {
      fireEvent.click(screen.getByText("Trigger Confirm"));
    });

    // Click No
    await act(async () => {
      fireEvent.click(screen.getByText("No"));
    });

    expect(onResult).toHaveBeenCalledWith(false);
  });
});
