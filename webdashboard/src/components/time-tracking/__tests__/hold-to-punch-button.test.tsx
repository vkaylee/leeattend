import { fireEvent, render, screen } from "@testing-library/react";

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { HoldToPunchButton } from "../hold-to-punch-button";

describe("HoldToPunchButton", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it("renders correctly with label", () => {
    render(<HoldToPunchButton isPunching={false} label="Hold to Punch" onPunch={vi.fn()} />);
    expect(screen.getByText("Hold to Punch")).toBeInTheDocument();
  });

  it("calls onPunch after holding for 1 second", () => {
    const mockOnPunch = vi.fn();
    render(<HoldToPunchButton isPunching={false} label="Hold to Punch" onPunch={mockOnPunch} />);
    
    const button = screen.getByRole("button");
    
    // Simulate pointer down
    fireEvent.pointerDown(button, { button: 0, pointerType: "mouse" });
    
    // Fast forward 500ms
    vi.advanceTimersByTime(500);
    expect(mockOnPunch).not.toHaveBeenCalled();
    
    // Fast forward another 520ms (total 1020ms) to ensure the 16ms interval fires
    vi.advanceTimersByTime(520);
    expect(mockOnPunch).toHaveBeenCalledTimes(1);
  });

  it("cancels punch if pointer is released early", () => {
    const mockOnPunch = vi.fn();
    render(<HoldToPunchButton isPunching={false} label="Hold to Punch" onPunch={mockOnPunch} />);
    
    const button = screen.getByRole("button");
    
    // Simulate pointer down
    fireEvent.pointerDown(button, { button: 0, pointerType: "mouse" });
    
    // Fast forward 500ms
    vi.advanceTimersByTime(500);
    
    // Release pointer
    fireEvent.pointerUp(button);
    
    // Fast forward remaining time
    vi.advanceTimersByTime(520);
    
    expect(mockOnPunch).not.toHaveBeenCalled();
  });

  it("does not start hold if isPunching is true", () => {
    const mockOnPunch = vi.fn();
    render(<HoldToPunchButton isPunching={true} label="Hold to Punch" onPunch={mockOnPunch} />);
    
    const button = screen.getByRole("button");
    
    fireEvent.pointerDown(button, { button: 0, pointerType: "mouse" });
    vi.advanceTimersByTime(1020);
    
    expect(mockOnPunch).not.toHaveBeenCalled();
    expect(button).toBeDisabled();
  });
});
