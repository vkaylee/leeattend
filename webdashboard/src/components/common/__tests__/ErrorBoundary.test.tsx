import React from "react";

import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { ErrorBoundary } from "../ErrorBoundary";

describe("ErrorBoundary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders children when no error occurs", () => {
    render(
      <ErrorBoundary>
        <div data-testid="child">Hello</div>
      </ErrorBoundary>,
    );
    expect(screen.getByTestId("child")).toHaveTextContent("Hello");
  });

  it("renders error UI when child throws", () => {
    const ThrowingComponent = () => {
      throw new Error("Test error message");
    };

    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>,
    );

    expect(screen.getByText("Something went wrong")).toBeInTheDocument();
    expect(screen.getByText("Test error message")).toBeInTheDocument();
  });

  it("renders Try again and Reload page buttons on error", () => {
    const ThrowingComponent = () => {
      throw new Error("boom");
    };

    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>,
    );

    expect(screen.getByText("Try again")).toBeInTheDocument();
    expect(screen.getByText("Reload page")).toBeInTheDocument();
  });

  it("retry button clears error and re-renders children", () => {
    let shouldThrow = true;
    const MaybeThrowing = () => {
      if (shouldThrow) throw new Error("first fail");
      return <div data-testid="recovered">Recovered</div>;
    };

    render(
      <ErrorBoundary>
        <MaybeThrowing />
      </ErrorBoundary>,
    );

    expect(screen.getByText("Something went wrong")).toBeInTheDocument();

    shouldThrow = false;
    fireEvent.click(screen.getByText("Try again"));

    expect(screen.getByTestId("recovered")).toHaveTextContent("Recovered");
  });

  it("uses custom fallback when provided", () => {
    const ThrowingComponent = () => {
      throw new Error("custom fallback test");
    };

    render(
      <ErrorBoundary fallback={<div data-testid="custom-fallback">Custom Error UI</div>}>
        <ThrowingComponent />
      </ErrorBoundary>,
    );

    expect(screen.getByTestId("custom-fallback")).toHaveTextContent("Custom Error UI");
  });

  it("calls onError callback when error occurs", () => {
    const onError = vi.fn();
    const ThrowingComponent = () => {
      throw new Error("callback test");
    };

    render(
      <ErrorBoundary onError={onError}>
        <ThrowingComponent />
      </ErrorBoundary>,
    );

    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError).toHaveBeenCalledWith(
      expect.objectContaining({ message: "callback test" }),
      expect.any(Object),
    );
  });

  it("recovers after clicking Try again (error state is reset)", () => {
    let shouldFail = true;
    const TransientFail = () => {
      if (shouldFail) throw new Error("transient");
      return <div data-testid="recovered">Loaded</div>;
    };

    render(
      <ErrorBoundary>
        <TransientFail />
      </ErrorBoundary>,
    );

    expect(screen.getByText("Something went wrong")).toBeInTheDocument();

    shouldFail = false;
    fireEvent.click(screen.getByText("Try again"));

    expect(screen.getByTestId("recovered")).toHaveTextContent("Loaded");
  });

  it("handles null children without crashing", () => {
    const { container } = render(
      <ErrorBoundary>{null}</ErrorBoundary>,
    );
    expect(container.textContent).toBe("");
  });
});
