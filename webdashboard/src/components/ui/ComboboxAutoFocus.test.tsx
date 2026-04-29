import React from "react";

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

vi.unmock("@/components/ui/Combobox");

import { Combobox } from "./Combobox";

import { Command } from "cmdk";

vi.mock("@radix-ui/react-popover", () => ({
  Root: ({ children, open, onOpenChange }: any) => (
    <div data-testid="popover-root" data-open={open} onClick={() => onOpenChange?.(!open)}>
      {children}
    </div>
  ),
  Trigger: ({ children }: any) => <div data-testid="popover-trigger">{children}</div>,
  Portal: ({ children }: any) => <div>{children}</div>,
  Content: ({ children }: any) => <div data-testid="popover-content">{children}</div>,
}));

vi.mock("cmdk", () => ({
  Command: Object.assign(
    ({ children, value, onValueChange }: any) => (
      <div data-testid="cmdk-root" data-value={value}>{children}</div>
    ),
    {
      Input: vi.fn(({ autoFocus, placeholder }: any) => (
        <input data-testid="cmdk-input" data-autofocus={autoFocus} placeholder={placeholder} />
      )),
      List: ({ children }: any) => <div>{children}</div>,
      Empty: ({ children }: any) => <div>{children}</div>,
      Group: ({ children }: any) => <div>{children}</div>,
      Item: ({ children, value }: any) => <div data-testid={`cmdk-item-${value}`}>{children}</div>,
    }
  ),
}));

describe("Combobox AutoFocus", () => {
  const options = [{ value: "1", label: "Option 1" }];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("starts with popover open when autoFocus is true", () => {
    render(<Combobox options={options} onChange={() => { }} autoFocus={true} />);
    const root = screen.getByTestId("popover-root");
    expect(root.getAttribute("data-open")).toBe("true");
  });

  it("always passes autoFocus=true to Command.Input", () => {
    render(<Combobox options={options} onChange={() => { }} />);
    expect(Command.Input).toHaveBeenCalledWith(
      expect.objectContaining({ autoFocus: true }),
      undefined
    );
  });
});

describe("Combobox Keyboard Shortcut", () => {
  const options = [
    { value: "1", label: "Alpha", description: "D001" },
    { value: "2", label: "Beta", description: "D002" },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("opens the popover when keyboard shortcut is pressed", () => {
    render(
      <Combobox
        options={options}
        onChange={() => { }}
        keyboardShortcut="/"
        onSearchChange={() => { }}
      />
    );

    // Initially closed (no autoFocus)
    const root = screen.getByTestId("popover-root");
    expect(root.getAttribute("data-open")).toBe("false");

    // Simulate pressing '/' on the document body
    fireEvent.keyDown(window, { key: "/" });

    // Should now be open
    expect(root.getAttribute("data-open")).toBe("true");
  });

  it("calls onSearchChange with empty string when shortcut is pressed", () => {
    const onSearchChange = vi.fn();
    render(
      <Combobox
        options={options}
        onChange={() => { }}
        keyboardShortcut="/"
        onSearchChange={onSearchChange}
      />
    );

    fireEvent.keyDown(window, { key: "/" });
    expect(onSearchChange).toHaveBeenCalledWith("");
  });

  it("does NOT trigger shortcut when typing in an input element", () => {
    const onSearchChange = vi.fn();
    render(
      <div>
        <input data-testid="other-input" />
        <Combobox
          options={options}
          onChange={() => { }}
          keyboardShortcut="/"
          onSearchChange={onSearchChange}
        />
      </div>
    );

    const otherInput = screen.getByTestId("other-input");
    fireEvent.keyDown(otherInput, { key: "/" });

    // Should NOT have been called because the event target is an input
    expect(onSearchChange).not.toHaveBeenCalled();
  });

  it("handles multiple shortcut keys when passed as an array", () => {
    const onSearchChange1 = vi.fn();
    const { unmount: unmount1 } = render(
      <Combobox
        options={options}
        onChange={() => { }}
        keyboardShortcut={["/", "Enter"]}
        onSearchChange={onSearchChange1}
      />
    );

    // Press Enter
    fireEvent.keyDown(window, { key: "Enter" });
    expect(onSearchChange1).toHaveBeenCalledTimes(1);
    unmount1();

    const onSearchChange2 = vi.fn();
    render(
      <Combobox
        options={options}
        onChange={() => { }}
        keyboardShortcut={["/", "Enter"]}
        onSearchChange={onSearchChange2}
      />
    );

    // Press /
    fireEvent.keyDown(window, { key: "/" });
    expect(onSearchChange2).toHaveBeenCalledTimes(1);
  });

  it("ignores shortcut keys when the combobox is already open", () => {
    const onSearchChange = vi.fn();
    render(
      <Combobox
        options={options}
        onChange={() => { }}
        keyboardShortcut="Enter"
        onSearchChange={onSearchChange}
        autoFocus={true} // Starts open
      />
    );

    const root = screen.getByTestId("popover-root");
    expect(root.getAttribute("data-open")).toBe("true");

    // Pressing Enter when already open should NOT trigger the shortcut logic
    // (because cmdk needs to handle the Enter key to select an item)
    fireEvent.keyDown(window, { key: "Enter" });

    // onSearchChange shouldn't be called by the shortcut handler
    expect(onSearchChange).not.toHaveBeenCalled();
  });

  it("clears the value when clearOnEscape is true and Escape is pressed while closed", () => {
    const onChange = vi.fn();
    const onSearchChange = vi.fn();

    render(
      <Combobox
        options={options}
        value="1"
        onChange={onChange}
        onSearchChange={onSearchChange}
        clearOnEscape={true}
      />
    );

    // Initial state is closed
    const root = screen.getByTestId("popover-root");
    expect(root.getAttribute("data-open")).toBe("false");

    // Press Escape
    fireEvent.keyDown(window, { key: "Escape" });

    // Both onChange and onSearchChange should be called to reset the value
    expect(onChange).toHaveBeenCalledWith("");
    expect(onSearchChange).toHaveBeenCalledWith("");
  });
});

describe("Combobox First Item Highlight", () => {
  it("sets Command value to first item's cmdk value", () => {
    const options = [
      { value: "emp-1", label: "Alice", description: "1541" },
      { value: "emp-2", label: "Bob", description: "1656" },
    ];

    render(<Combobox options={options} onChange={() => { }} />);

    const cmdkRoot = screen.getByTestId("cmdk-root");
    // The cmdk value should be constructed from the first option
    expect(cmdkRoot.getAttribute("data-value")).toBe("Alice 1541 emp-1");
  });

  it("updates highlighted value when options change", () => {
    const options1 = [
      { value: "emp-1", label: "Alice", description: "1541" },
    ];
    const options2 = [
      { value: "emp-3", label: "Charlie", description: "9999" },
    ];

    const { rerender } = render(<Combobox options={options1} onChange={() => { }} />);

    let cmdkRoot = screen.getByTestId("cmdk-root");
    expect(cmdkRoot.getAttribute("data-value")).toBe("Alice 1541 emp-1");

    // Re-render with new options (simulating server-side search returning different results)
    rerender(<Combobox options={options2} onChange={() => { }} />);

    cmdkRoot = screen.getByTestId("cmdk-root");
    expect(cmdkRoot.getAttribute("data-value")).toBe("Charlie 9999 emp-3");
  });
});
