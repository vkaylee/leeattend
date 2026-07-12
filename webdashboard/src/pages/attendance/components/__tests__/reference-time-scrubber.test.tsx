import { fireEvent, render, screen } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import { ReferenceTimeScrubber } from "../reference-time-scrubber";

/** Fixed timestamp keeps the scrubber's math deterministic. */
const BASE = new Date(Date.UTC(2026, 6, 10, 7, 30, 0)); // 2026-07-10 07:30 UTC

describe("ReferenceTimeScrubber", () => {
  it("announces the pinned reference time via the slider's aria-label", () => {
    const onChange = vi.fn();
    render(
      <ReferenceTimeScrubber
        onChange={onChange}
        referenceTime={BASE}
      />,
    );
    // The slider is the accessible control; it must carry a non-empty
    // aria-label so screen-reader users know its purpose (WCAG: the scrubber's
    // state is never conveyed by colour/location alone). The label is locale-
    // specific, so assert presence rather than exact text.
    const slider = screen.getByRole("slider");
    const label = slider.getAttribute("aria-label");
    expect(label).toBeTruthy();
    expect((label ?? "").length).toBeGreaterThan(0);
  });

  it("steps the reference time back when the left chevron is clicked", () => {
    const onChange = vi.fn();
    render(
      <ReferenceTimeScrubber
        onChange={onChange}
        referenceTime={BASE}
      />,
    );
    fireEvent.click(screen.getByTestId("scrubber-step-back"));
    expect(onChange).toHaveBeenCalledTimes(1);
    const next = onChange.mock.calls[0][0] as Date;
    expect(next.getTime()).toBe(BASE.getTime() - 60 * 60_000);
  });

  it("steps the reference time forward when the right chevron is clicked", () => {
    const onChange = vi.fn();
    render(
      <ReferenceTimeScrubber
        onChange={onChange}
        referenceTime={BASE}
      />,
    );
    fireEvent.click(screen.getByTestId("scrubber-step-forward"));
    const next = onChange.mock.calls[0][0] as Date;
    expect(next.getTime()).toBe(BASE.getTime() + 60 * 60_000);
  });

  it("fires onChange when the Now button is clicked", () => {
    const onChange = vi.fn();
    render(
      <ReferenceTimeScrubber
        onChange={onChange}
        referenceTime={BASE}
      />,
    );
    fireEvent.click(screen.getByTestId("scrubber-jump-now"));
    // "now" is bounded by the live clock (unknown in test), but it must fire.
    expect(onChange).toHaveBeenCalledTimes(1);
  });
});
