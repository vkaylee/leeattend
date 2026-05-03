import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";

import SearchBox from "../search-box";

describe("SearchBox", () => {
    it("renders with placeholder text", () => {
        render(<SearchBox onChange={() => { /* empty */ }} placeholder="Search employees..." value="" />);
        expect(screen.getByPlaceholderText("Search employees...")).toBeDefined();
    });

    it("renders the search icon", () => {
        const { container } = render(<SearchBox onChange={() => { /* empty */ }} value="" />);
        const svg = container.querySelector("svg");
        expect(svg).toBeDefined();
    });

    it("calls onChange when typing", async () => {
        const onChange = vi.fn();
        const user = userEvent.setup();
        render(<SearchBox data-testid="search" onChange={onChange} value="" />);

        const input = screen.getByTestId("search");
        await user.type(input, "A");
        expect(onChange).toHaveBeenCalledWith("A");
    });

    it("shows clear X button when value is non-empty", () => {
        const { container } = render(<SearchBox onChange={() => { /* empty */ }} value="hello" />);
        // X button should be present as a button inside the component
        const buttons = container.querySelectorAll("button");
        expect(buttons.length).toBe(1);
    });

    it("does NOT show clear X button when value is empty", () => {
        const { container } = render(<SearchBox onChange={() => { /* empty */ }} value="" />);
        const buttons = container.querySelectorAll("button");
        expect(buttons.length).toBe(0);
    });

    it("calls onChange with empty string when X is clicked", async () => {
        const onChange = vi.fn();
        const user = userEvent.setup();
        const { container } = render(<SearchBox onChange={onChange} value="hello" />);

        const clearButton = container.querySelector("button")!;
        await user.click(clearButton);
        expect(onChange).toHaveBeenCalledWith("");
    });

    it("calls select() on focus to select all text", async () => {
        const selectSpy = vi.spyOn(HTMLInputElement.prototype, "select");
        const user = userEvent.setup();
 
        render(<SearchBox data-testid="search" onChange={() => { /* empty */ }} value="hello" />);

        const input = screen.getByTestId("search");
        await user.click(input);

        expect(selectSpy).toHaveBeenCalled();
        selectSpy.mockRestore();
    });

    it("applies custom className", () => {
        const { container } = render(<SearchBox className="custom-class" onChange={() => { /* empty */ }} value="" />);
        const wrapper = container.firstChild as HTMLElement;
        expect(wrapper.className).toContain("custom-class");
    });
});
