import React from "react";

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import ConfirmDialog from "../ConfirmDialog";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
 
    t: (key: string, options?: { defaultValue?: string }) => {
      if (options && options.defaultValue) {
        return options.defaultValue;
      }
      return key;
    },
  }),
}));

describe("ConfirmDialog Component", () => {
  it("renders correctly with given props", () => {
    render(
      <ConfirmDialog
        isOpen={true}
        onClose={() => {}}
        onConfirm={() => {}}
        title="Test Confirm"
        message="Are you sure you want to proceed?"
        confirmLabel="Yes, Do It"
        cancelLabel="No, Wait"
      />
    );

    expect(screen.getByText("Test Confirm")).toBeInTheDocument();
    expect(screen.getByText("Are you sure you want to proceed?")).toBeInTheDocument();
    expect(screen.getByText("Yes, Do It")).toBeInTheDocument();
    expect(screen.getByText("No, Wait")).toBeInTheDocument();
  });

  it("calls onClose when cancel button is clicked", () => {
    const onClose = vi.fn();
    render(
      <ConfirmDialog
        isOpen={true}
        onClose={onClose}
        onConfirm={() => {}}
        title="Test Confirm"
        message="Are you sure?"
      />
    );

    fireEvent.click(screen.getByText("Cancel"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("calls onConfirm when confirm button is clicked", async () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        isOpen={true}
        onClose={() => {}}
        onConfirm={onConfirm}
        title="Test Confirm"
        message="Are you sure?"
      />
    );

    fireEvent.click(screen.getByText("Confirm"));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("disables buttons when isLoading is true", () => {
    render(
      <ConfirmDialog
        isOpen={true}
        onClose={() => {}}
        onConfirm={() => {}}
        title="Test Confirm"
        message="Are you sure?"
        isLoading={true}
      />
    );

    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    const cancelButton = screen.getByRole("button", { name: "Cancel" });

    expect(confirmButton).toBeDisabled();
    expect(cancelButton).toBeDisabled();
  });
});
