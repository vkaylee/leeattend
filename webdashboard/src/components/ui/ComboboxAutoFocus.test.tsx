import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import React from "react";

vi.unmock("@/components/ui/Combobox");

import { Combobox } from "./Combobox";
import { Command } from "cmdk";

vi.mock("@radix-ui/react-popover", () => ({
  Root: ({ children, open }: any) => <div data-testid="popover-root" data-open={open}>{children}</div>,
  Trigger: ({ children }: any) => <div data-testid="popover-trigger">{children}</div>,
  Portal: ({ children }: any) => <div>{children}</div>,
  Content: ({ children }: any) => <div data-testid="popover-content">{children}</div>,
}));

vi.mock("cmdk", () => ({
  Command: Object.assign(
    ({ children }: any) => <div>{children}</div>,
    {
      Input: vi.fn(({ autoFocus, placeholder }: any) => (
        <input data-testid="cmdk-input" data-autofocus={autoFocus} placeholder={placeholder} />
      )),
      List: ({ children }: any) => <div>{children}</div>,
      Empty: ({ children }: any) => <div>{children}</div>,
      Group: ({ children }: any) => <div>{children}</div>,
      Item: ({ children }: any) => <div>{children}</div>,
    }
  ),
}));

describe("Combobox AutoFocus", () => {
  const options = [{ value: "1", label: "Option 1" }];

  it("starts with popover open when autoFocus is true", () => {
    render(<Combobox options={options} onChange={() => {}} autoFocus={true} />);
    const root = screen.getByTestId("popover-root");
    expect(root.getAttribute("data-open")).toBe("true");
  });

  it("passes autoFocus to Command.Input", () => {
    render(<Combobox options={options} onChange={() => {}} autoFocus={true} />);
    // The second argument in received call is 'undefined', but toHaveBeenCalledWith(matcher) checks the first arg.
    // If there's only one matcher, it expects only one argument.
    // We'll use toHaveBeenCalledWith(expect.objectContaining({ autoFocus: true }), undefined)
    expect(Command.Input).toHaveBeenCalledWith(
      expect.objectContaining({ autoFocus: true }),
      undefined
    );
  });
});
