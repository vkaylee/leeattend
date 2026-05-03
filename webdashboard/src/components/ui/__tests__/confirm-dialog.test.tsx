import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import ConfirmDialog from "../confirm-dialog";

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

describe("ConfirmDialog Component", () => {
  it("renders correctly with given props", () => {
    render(
      <ConfirmDialog
        cancelLabel="No, Wait"
        confirmLabel="Yes, Do It"
        isOpen={true}
        message="Are you sure you want to proceed?"
        onClose={() => { /* empty */ }}
        onConfirm={() => { /* empty */ }}
        title="Test Confirm"
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
        message="Are you sure?"
        onClose={onClose}
        onConfirm={() => { /* empty */ }}
        title="Test Confirm"
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
        message="Are you sure?"
        onClose={() => { /* empty */ }}
        onConfirm={onConfirm}
        title="Test Confirm"
      />
    );

    fireEvent.click(screen.getByText("Confirm"));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("disables buttons when isLoading is true", () => {
    render(
      <ConfirmDialog
        isLoading={true}
        isOpen={true}
        message="Are you sure?"
        onClose={() => { /* empty */ }}
        onConfirm={() => { /* empty */ }}
        title="Test Confirm"
      />
    );

    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    const cancelButton = screen.getByRole("button", { name: "Cancel" });

    expect(confirmButton).toBeDisabled();
    expect(cancelButton).toBeDisabled();
  });
});
