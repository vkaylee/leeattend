import React from "react";

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";

import SearchBox from "../SearchBox";

describe("SearchBox", () => {
    it("renders with placeholder text", () => {
        render(<SearchBox value="" onChange={() => { }} placeholder="Search employees..." />);
        expect(screen.getByPlaceholderText("Search employees...")).toBeDefined();
    });

    it("renders the search icon", () => {
        const { container } = render(<SearchBox value="" onChange={() => { }} />);
        const svg = container.querySelector("svg");
        expect(svg).toBeDefined();
    });

    it("calls onChange when typing", async () => {
        const onChange = vi.fn();
        const user = userEvent.setup();
        render(<SearchBox value="" onChange={onChange} data-testid="search" />);

        const input = screen.getByTestId("search");
        await user.type(input, "A");
        expect(onChange).toHaveBeenCalledWith("A");
    });

    it("shows clear X button when value is non-empty", () => {
        const { container } = render(<SearchBox value="hello" onChange={() => { }} />);
        // X button should be present as a button inside the component
        const buttons = container.querySelectorAll("button");
        expect(buttons.length).toBe(1);
    });

    it("does NOT show clear X button when value is empty", () => {
        const { container } = render(<SearchBox value="" onChange={() => { }} />);
        const buttons = container.querySelectorAll("button");
        expect(buttons.length).toBe(0);
    });

    it("calls onChange with empty string when X is clicked", async () => {
        const onChange = vi.fn();
        const user = userEvent.setup();
        const { container } = render(<SearchBox value="hello" onChange={onChange} />);

        const clearBtn = container.querySelector("button")!;
        await user.click(clearBtn);
        expect(onChange).toHaveBeenCalledWith("");
    });

    it("calls select() on focus to select all text", async () => {
        const selectSpy = vi.spyOn(HTMLInputElement.prototype, "select");
        const user = userEvent.setup();
 
        render(<SearchBox value="hello" onChange={() => { }} data-testid="search" />);

        const input = screen.getByTestId("search");
        await user.click(input);

        expect(selectSpy).toHaveBeenCalled();
        selectSpy.mockRestore();
    });

    it("applies custom className", () => {
        const { container } = render(<SearchBox value="" onChange={() => { }} className="custom-class" />);
        const wrapper = container.firstChild as HTMLElement;
        expect(wrapper.className).toContain("custom-class");
    });
});
