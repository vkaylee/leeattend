import React from "react";

import { render, _screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";

import FloatingRecordBadge from "../FloatingRecordBadge";

describe("FloatingRecordBadge", () => {
    it("renders loaded and total counts", () => {
        const { container } = render(<FloatingRecordBadge loaded={5} total={20} />);
        expect(container.textContent).toContain("5");
        expect(container.textContent).toContain("20");
    });

    it("renders custom label", () => {
        const { container } = render(
            <FloatingRecordBadge loaded={3} total={10} label="items" />
        );
        expect(container.textContent).toContain("items");
    });

    it("renders default 'records' label", () => {
        const { container } = render(
            <FloatingRecordBadge loaded={1} total={1} />
        );
        expect(container.textContent).toContain("records");
    });

    it("uses fixed bottom-6 positioning", () => {
        const { container } = render(<FloatingRecordBadge loaded={1} total={1} />);
        const wrapper = container.firstChild as HTMLElement;
        expect(wrapper.className).toContain("fixed");
        expect(wrapper.className).toContain("bottom-6");
    });

    it("renders default variant with slate/primary colors", () => {
        const { container } = render(<FloatingRecordBadge loaded={1} total={1} />);
        const badge = container.querySelector(".border-slate-200");
        expect(badge).toBeDefined();
    });

    it("renders amber variant with amber colors", () => {
        const { container } = render(
            <FloatingRecordBadge loaded={1} total={1} variant="amber" />
        );
        const badge = container.querySelector(".border-amber-200");
        expect(badge).toBeDefined();
    });
});
