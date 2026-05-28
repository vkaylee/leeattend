import { render, screen } from "@testing-library/react";
import React from "react";
import { describe, it, expect } from "vitest";

import Button from "../button";

describe("Button Component", () => {
  it("renders with default props", () => {
    render(<Button>Click me</Button>);
    const button = screen.getByRole("button", { name: /click me/i });
    expect(button).toBeInTheDocument();
  });

  describe("Aspect Ratio Enforcement (via CSS Classes)", () => {
    // Note: Vitest uses JSDOM which doesn't compute real CSS layout dimensions.
    // Therefore, we cannot test boundingBox width/height directly here like in Playwright.
    // Instead, we verify that the correct Tailwind classes that enforce the aspect ratio are applied.

    it("applies min-width classes for size 'sm'", () => {
      render(<Button size="sm">Small</Button>);
      const button = screen.getByRole("button", { name: /small/i });
      expect(button).toHaveClass("h-[var(--ui-height-sm)]");
      expect(button).toHaveClass("min-w-[var(--ui-height-sm)]");
    });

    it("applies min-width classes for size 'md' (default)", () => {
      render(<Button>Medium</Button>);
      const button = screen.getByRole("button", { name: /medium/i });
      expect(button).toHaveClass("h-[var(--ui-height-md)]");
      expect(button).toHaveClass("min-w-[var(--ui-height-md)]");
    });

    it("applies min-width classes for size 'lg'", () => {
      render(<Button size="lg">Large</Button>);
      const button = screen.getByRole("button", { name: /large/i });
      expect(button).toHaveClass("h-[var(--ui-height-lg)]");
      expect(button).toHaveClass("min-w-[var(--ui-height-lg)]");
    });

    it("applies fixed width/height for size 'icon'", () => {
      render(<Button size="icon">Icon</Button>);
      const button = screen.getByRole("button", { name: /icon/i });
      expect(button).toHaveClass("h-[var(--ui-height-md)]");
      expect(button).toHaveClass("w-[var(--ui-height-md)]");
    });
  });

  describe("Accessibility & Contrast", () => {
    it("applies correct high-contrast grey classes when disabled to pass WCAG AA", () => {
      render(<Button disabled>Disabled Button</Button>);
      const button = screen.getByRole("button", { name: /disabled button/i });
      
      expect(button).toBeDisabled();
      // Should have the explicit disabled styling rather than opacity-50
      expect(button).toHaveClass("disabled:bg-slate-200");
      expect(button).toHaveClass("disabled:text-slate-600");
      expect(button).toHaveClass("dark:disabled:bg-slate-800");
      expect(button).toHaveClass("dark:disabled:text-slate-400");
    });

    it("uses cursor-not-allowed when disabled for better UX", () => {
      render(<Button disabled>Disabled UX</Button>);
      const button = screen.getByRole("button", { name: /disabled ux/i });
      expect(button).toHaveClass("disabled:cursor-not-allowed");
      expect(button).not.toHaveClass("disabled:pointer-events-none"); // ensure we don't use this
    });
  });

  describe("Loading State", () => {
    it("disables the button when loading is true", () => {
      render(<Button loading>Loading Button</Button>);
      // Cannot query by name easily if loader replaces content or adds icon,
      // but role="button" should be disabled.
      const button = screen.getByRole("button");
      expect(button).toBeDisabled();
    });
  });
});
