import "@testing-library/jest-dom";
import { fireEvent, render, screen } from "@testing-library/react";
import React from "react";
import { describe, expect, it } from "vitest";

import { useHorizontalDragScroll } from "../use-horizontal-drag-scroll";

function TestScroller({ enabled = true }: { enabled?: boolean }) {
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>({ enabled });

  return (
    <div
      data-testid="scroller"
      onClickCapture={handleHorizontalDragClick}
      ref={setHorizontalDragScrollRef}
    >
      <a href="/action">Action</a>
      <span>Scrollable content</span>
    </div>
  );
}

function setOverflow(element: HTMLElement, values: { clientWidth: number; scrollWidth: number }) {
  Object.defineProperties(element, {
    clientWidth: { configurable: true, value: values.clientWidth },
    scrollWidth: { configurable: true, value: values.scrollWidth },
  });
}

describe("useHorizontalDragScroll", () => {
  it("pans horizontally after horizontal intent crosses the threshold", () => {
    render(<TestScroller />);
    const scroller = screen.getByTestId("scroller");
    setOverflow(scroller, { clientWidth: 100, scrollWidth: 500 });
    scroller.scrollLeft = 80;

    fireEvent.pointerDown(scroller, {
      button: 0,
      clientX: 80,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: 50,
      clientY: 21,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });

    expect(scroller.scrollLeft).toBe(110);
  });

  it("keeps a click and vertical movement out of horizontal scrolling", () => {
    render(<TestScroller />);
    const scroller = screen.getByTestId("scroller");
    const clickTarget = screen.getByText("Scrollable content");
    setOverflow(scroller, { clientWidth: 100, scrollWidth: 500 });

    fireEvent.pointerDown(scroller, {
      button: 0,
      clientX: 20,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: 21,
      clientY: 40,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.click(clickTarget);

    expect(scroller.scrollLeft).toBe(0);
  });

  it("does not steal interactive controls or non-primary pointers", () => {
    render(<TestScroller />);
    const scroller = screen.getByTestId("scroller");
    const button = screen.getByRole("link", { name: "Action" });
    setOverflow(scroller, { clientWidth: 100, scrollWidth: 500 });

    fireEvent.pointerDown(button, {
      button: 0,
      clientX: 20,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: 0,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.pointerDown(scroller, {
      button: 0,
      clientX: 20,
      clientY: 20,
      isPrimary: false,
      pointerId: 2,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: 0,
      clientY: 20,
      isPrimary: false,
      pointerId: 2,
      pointerType: "mouse",
    });

    expect(scroller.scrollLeft).toBe(0);
  });

  it("clamps panning to both horizontal scroll boundaries", () => {
    render(<TestScroller />);
    const scroller = screen.getByTestId("scroller");
    setOverflow(scroller, { clientWidth: 100, scrollWidth: 500 });

    fireEvent.pointerDown(scroller, {
      button: 0,
      clientX: 20,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: 200,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    expect(scroller.scrollLeft).toBe(0);

    fireEvent.pointerUp(window, { isPrimary: true, pointerId: 1, pointerType: "mouse" });
    scroller.scrollLeft = 100;
    fireEvent.pointerDown(scroller, {
      button: 0,
      clientX: 20,
      clientY: 20,
      isPrimary: true,
      pointerId: 2,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: -500,
      clientY: 20,
      isPrimary: true,
      pointerId: 2,
      pointerType: "mouse",
    });

    expect(scroller.scrollLeft).toBe(400);
  });

  it("does not pan when disabled", () => {
    render(<TestScroller enabled={false} />);
    const scroller = screen.getByTestId("scroller");
    setOverflow(scroller, { clientWidth: 100, scrollWidth: 500 });

    fireEvent.pointerDown(scroller, {
      button: 0,
      clientX: 20,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });
    fireEvent.pointerMove(window, {
      clientX: -100,
      clientY: 20,
      isPrimary: true,
      pointerId: 1,
      pointerType: "mouse",
    });

    expect(scroller.scrollLeft).toBe(0);
  });
});
