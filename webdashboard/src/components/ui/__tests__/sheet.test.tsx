import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

import Sheet from "../sheet";

describe("Sheet", () => {
    const defaultProps = {
        isOpen: true,
        onClose: vi.fn(),
        title: "Test Sheet",
        children: <div>Sheet content</div>,
    };

    it("does not render when closed and not visible", () => {
        render(<Sheet {...defaultProps} isOpen={false} />);
        expect(screen.queryByText("Test Sheet")).toBeNull();
    });

    it("renders title and children when open", () => {
        render(<Sheet {...defaultProps} />);
        expect(screen.getByText("Test Sheet")).toBeInTheDocument();
        expect(screen.getByText("Sheet content")).toBeInTheDocument();
    });

    it("names the dialog from its visible title", () => {
        render(<Sheet {...defaultProps} />);

        const dialog = screen.getByRole("dialog", { name: "Test Sheet" });
        const titleId = dialog.getAttribute("aria-labelledby");

        expect(titleId).toBeTruthy();
        expect(document.getElementById(titleId!)).toHaveTextContent("Test Sheet");
    });

    it("preserves an explicit accessible title id", () => {
        render(<Sheet {...defaultProps} ariaLabelledBy="custom-title" />);

        expect(screen.getByTestId("sheet-panel")).toHaveAttribute("aria-labelledby", "custom-title");
    });

    it("renders subtitle when provided", () => {
        render(<Sheet {...defaultProps} subtitle="Sub text" />);
        expect(screen.getByText("Sub text")).toBeInTheDocument();
    });

    it("renders headerExtra when provided", () => {
        render(<Sheet {...defaultProps} headerExtra={<button>Extra</button>} />);
        expect(screen.getByText("Extra")).toBeInTheDocument();
    });

    it("calls onClose when close button is clicked", () => {
        const onClose = vi.fn();
        render(<Sheet {...defaultProps} onClose={onClose} />);
        const closeButton = screen.getByRole("button");
        fireEvent.click(closeButton);
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("calls onClose when backdrop is clicked", () => {
        const onClose = vi.fn();
        render(<Sheet {...defaultProps} onClose={onClose} />);
        // Backdrop is the first div with opacity class
        const backdrop = document.querySelector(".backdrop-blur-lg");
        expect(backdrop).not.toBeNull();
        fireEvent.click(backdrop!);
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("applies className to sheet panel", () => {
        render(<Sheet {...defaultProps} className="custom-class" />);
        const panel = screen.getByTestId("sheet-panel");
        expect(panel.className).toContain("custom-class");
    });

    it("applies contentClassName to content area, overriding defaults", () => {
        render(<Sheet {...defaultProps} contentClassName="!overflow-hidden" />);
        const contentArea = screen.getByText("Sheet content").parentElement?.parentElement;
        expect(contentArea).not.toBeNull();
        expect(contentArea!.className).toContain("!overflow-hidden");
        // Default overflow-y-auto still present in class list, but !overflow-hidden takes precedence
        expect(contentArea!.className).toContain("overflow-y-auto");
    });

    it("content area has default overflow-y-auto when no contentClassName is provided", () => {
        render(<Sheet {...defaultProps} />);
        const contentArea = screen.getByText("Sheet content").parentElement?.parentElement;
        expect(contentArea).not.toBeNull();
        expect(contentArea!.className).toContain("overflow-y-auto");
        expect(contentArea!.className).toContain("min-h-0");
        expect(contentArea!.className).not.toContain("!overflow-hidden");
    });

    it("inner content wrapper has h-full for flex children", () => {
        render(<Sheet {...defaultProps} />);
        // Inner wrapper must participate in flex sizing for nested scroll owners.
        const innerWrapper = screen.getByText("Sheet content").parentElement;
        expect(innerWrapper).not.toBeNull();
        expect(innerWrapper!.className).toContain("flex-1");
        expect(innerWrapper!.className).toContain("min-h-0");
        expect(innerWrapper!.className).toContain("flex-col");
    });

    it("panel has flex-col layout for header/content stacking", () => {
        render(<Sheet {...defaultProps} />);
        const panel = screen.getByTestId("sheet-panel");
        expect(panel.className).toContain("flex");
        expect(panel.className).toContain("flex-col");
        expect(panel.className).toContain("h-full");
    });

    it("renders on left side when side='left'", () => {
        render(<Sheet {...defaultProps} side="left" />);
        const panel = screen.getByTestId("sheet-panel");
        expect(panel.className).toContain("rounded-r-[3rem]");
    });

    it("renders on right side by default", () => {
        render(<Sheet {...defaultProps} />);
        const panel = screen.getByTestId("sheet-panel");
        expect(panel.className).toContain("rounded-l-[3rem]");
    });

    it("calls onClose when Escape key is pressed", () => {
        const onClose = vi.fn();
        render(<Sheet {...defaultProps} onClose={onClose} />);

        fireEvent.keyDown(window, { key: "Escape" });

        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("does not call onClose when other keys are pressed", () => {
        const onClose = vi.fn();
        render(<Sheet {...defaultProps} onClose={onClose} />);

        fireEvent.keyDown(window, { key: "Enter" });

        expect(onClose).not.toHaveBeenCalled();
    });
});
