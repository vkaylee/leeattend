import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { NewVersionBanner } from "../NewVersionBanner";

describe("NewVersionBanner", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders Refresh and Dismiss buttons", () => {
    render(<NewVersionBanner onDismiss={vi.fn()} onRefresh={vi.fn()} />);

    expect(screen.getByText("A new version is available")).toBeInTheDocument();
    expect(screen.getByText("Refresh")).toBeInTheDocument();
    expect(screen.getByLabelText("Dismiss")).toBeInTheDocument();
  });

  it("calls onRefresh after animation when Refresh is clicked", () => {
    const onRefresh = vi.fn();
    render(<NewVersionBanner onDismiss={vi.fn()} onRefresh={onRefresh} />);

    fireEvent.click(screen.getByText("Refresh"));

    expect(onRefresh).not.toHaveBeenCalled();
    vi.advanceTimersByTime(300);
    expect(onRefresh).toHaveBeenCalledTimes(1);
  });

  it("calls onDismiss after animation when Dismiss is clicked", () => {
    const onDismiss = vi.fn();
    render(<NewVersionBanner onDismiss={onDismiss} onRefresh={vi.fn()} />);

    fireEvent.click(screen.getByLabelText("Dismiss"));

    expect(onDismiss).not.toHaveBeenCalled();
    vi.advanceTimersByTime(300);
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it("has translate-y-0 class on mount", () => {
    render(<NewVersionBanner onDismiss={vi.fn()} onRefresh={vi.fn()} />);

    const banner = screen.getByText("A new version is available").closest("[class*='fixed']");
    expect(banner?.className).toContain("translate-y-0");
  });

  it("applies slide-out animation on dismiss click", () => {
    render(<NewVersionBanner onDismiss={vi.fn()} onRefresh={vi.fn()} />);

    fireEvent.click(screen.getByLabelText("Dismiss"));

    const banner = screen.getByText("A new version is available").closest("[class*='fixed']");
    expect(banner?.className).toContain("-translate-y-full");
  });
});
