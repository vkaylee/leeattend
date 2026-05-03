import { render, screen } from "@testing-library/react";
import React, { createRef } from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import type { NavigableListColumn } from "../navigable-list";

import NavigableList from "../navigable-list";

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
    observe() { /* empty */ }
    unobserve() { /* empty */ }
    disconnect() { /* empty */ }
}
 
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock scroll methods not available in jsdom
Element.prototype.scrollIntoView = vi.fn();
 
Element.prototype.scrollTo = vi.fn();

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback ?? key,
        i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
    }),
}));

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

interface TestItem {
    id: string;
    name: string;
    value: number;
}

const items: TestItem[] = [
    { id: "1", name: "Alpha", value: 10 },
    { id: "2", name: "Beta", value: 20 },
    { id: "3", name: "Charlie", value: 30 },
];

const columns: NavigableListColumn<TestItem>[] = [
    { key: "name", header: "Name", cell: (item) => item.name },
    { key: "value", header: "Value", cell: (item) => `${item.value}`, cellClassName: "text-right" },
];

const defaultProps = {
    columns,
    items,
    itemKey: (item: TestItem) => item.id,
    onSelect: vi.fn(),
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("NavigableList", () => {
    beforeEach(() => { vi.clearAllMocks(); });

    // ── Rendering ──────────────────────────────────────────────

    it("renders all items", () => {
        render(<NavigableList {...defaultProps} />);
        expect(screen.getByText("Alpha")).toBeDefined();
        expect(screen.getByText("Beta")).toBeDefined();
        expect(screen.getByText("Charlie")).toBeDefined();
    });

    it("renders column headers", () => {
        render(<NavigableList {...defaultProps} />);
        expect(screen.getByText("Name")).toBeDefined();
        expect(screen.getByText("Value")).toBeDefined();
    });

    it("renders empty state when no items", () => {
        render(<NavigableList {...defaultProps} emptyMessage="Nothing here" items={[]} />);
        expect(screen.getByText("Nothing here")).toBeDefined();
    });

    it("renders loading skeletons when isLoading", () => {
        const { container } = render(<NavigableList {...defaultProps} isLoading skeletonRows={3} />);
        const skeletons = container.querySelectorAll(".animate-pulse");
        // 3 skeleton rows × 2 columns = 6 pulse divs
        expect(skeletons.length).toBe(6);
    });

    it("hides columns with hidden=true", () => {
        const cols: NavigableListColumn<TestItem>[] = [
            ...columns,
            { key: "secret", header: "Secret", cell: () => "hidden", hidden: true },
        ];
        render(<NavigableList {...defaultProps} columns={cols} />);
        expect(screen.queryByText("Secret")).toBeNull();
        expect(screen.queryByText("hidden")).toBeNull();
    });

    // ── Search ─────────────────────────────────────────────────

    it("renders search box when onSearchChange is provided", () => {
        render(
            <NavigableList
                {...defaultProps}
                onSearchChange={vi.fn()}
                searchTestId="test-search"
                searchValue=""
            />
        );
        expect(screen.getByTestId("test-search")).toBeDefined();
    });

    it("does not render search box when onSearchChange is omitted", () => {
        render(<NavigableList {...defaultProps} />);
        expect(screen.queryByTestId("navigable-list-search")).toBeNull();
    });

    // ── Row interaction ────────────────────────────────────────

    it("calls onSelect when a row is clicked", async () => {
        const onSelect = vi.fn();
        const user = userEvent.setup();
        render(<NavigableList {...defaultProps} onSelect={onSelect} />);

        await user.click(screen.getByText("Beta"));
        expect(onSelect).toHaveBeenCalledWith(items[1]);
    });

    it("highlights selected row", () => {
        const { container } = render(
            <NavigableList {...defaultProps} selectedKey="2" />
        );
        const row = container.querySelector("#nav-row-1");
        expect(row?.className).toContain("bg-primary-50");
    });

    it("shows active bar on selected row when showActiveBar is true", () => {
        const { container } = render(
            <NavigableList {...defaultProps} selectedKey="2" showActiveBar />
        );
        const bar = container.querySelector(".bg-primary-600.rounded-r-full");
        expect(bar).toBeDefined();
    });

    // ── Keyboard navigation ────────────────────────────────────

    it("ArrowDown moves focus to the next row", async () => {
        const user = userEvent.setup();
        const { container } = render(
            <NavigableList
                {...defaultProps}
                onSearchChange={vi.fn()}
                searchTestId="test-search"
                searchValue=""
            />
        );

        const searchInput = screen.getByTestId("test-search");
        await user.click(searchInput);
        await user.keyboard("{ArrowDown}");

        // Focus should move from 0 to 1 — row 1 should have focused class
        const row1 = container.querySelector("#nav-row-1");
        expect(row1?.className).toContain("ring-primary-500");
    });

    it("ArrowUp moves focus to the previous row", async () => {
        const user = userEvent.setup();
        const { container } = render(
            <NavigableList
                {...defaultProps}
                onSearchChange={vi.fn()}
                searchTestId="test-search"
                searchValue=""
            />
        );

        const searchInput = screen.getByTestId("test-search");
        await user.click(searchInput);
        // Go down twice, then up once → should be at index 1
        await user.keyboard("{ArrowDown}{ArrowDown}{ArrowUp}");

        const row1 = container.querySelector("#nav-row-1");
        expect(row1?.className).toContain("ring-primary-500");
    });

    it("Enter calls onSelect with the focused item", async () => {
        const onSelect = vi.fn();
        const user = userEvent.setup();
        render(
            <NavigableList
                {...defaultProps}
                onSearchChange={vi.fn()}
                onSelect={onSelect}
                searchTestId="test-search"
                searchValue=""
            />
        );

        const searchInput = screen.getByTestId("test-search");
        await user.click(searchInput);
        await user.keyboard("{ArrowDown}{Enter}");

        expect(onSelect).toHaveBeenCalledWith(items[1]);
    });

    it("ArrowDown does not go past the last item", async () => {
        const user = userEvent.setup();
        const { container } = render(
            <NavigableList
                {...defaultProps}
                onSearchChange={vi.fn()}
                searchTestId="test-search"
                searchValue=""
            />
        );

        const searchInput = screen.getByTestId("test-search");
        await user.click(searchInput);
        // Press ArrowDown 10 times (more than items.length)
        for (let index = 0; index < 10; index++) {
            await user.keyboard("{ArrowDown}");
        }

        // Should be clamped at last index (2)
        const lastRow = container.querySelector("#nav-row-2");
        expect(lastRow?.className).toContain("ring-primary-500");
    });

    it("ArrowUp does not go below 0", async () => {
        const user = userEvent.setup();
        const { container } = render(
            <NavigableList
                {...defaultProps}
                onSearchChange={vi.fn()}
                searchTestId="test-search"
                searchValue=""
            />
        );

        const searchInput = screen.getByTestId("test-search");
        await user.click(searchInput);
        // Press ArrowUp when already at 0 — should stay at 0
        await user.keyboard("{ArrowUp}{ArrowUp}");

        // Row 0 should have focus ring (ArrowUp sets hasNavigated=true)
        const firstRow = container.querySelector("#nav-row-0");
        expect(firstRow?.className).toContain("ring-primary-500");
        // Row 1 should NOT have focus ring
        const secondRow = container.querySelector("#nav-row-1");
        expect(secondRow?.className).not.toContain("ring-primary-500");
    });

    // ── Sortable headers ───────────────────────────────────────

    it("calls onHeaderClick when a sortable header is clicked", async () => {
        const onSort = vi.fn();
        const user = userEvent.setup();
        const sortableCols: NavigableListColumn<TestItem>[] = [
            { key: "name", header: "Name", cell: (item) => item.name, onHeaderClick: onSort },
            { key: "value", header: "Value", cell: (item) => `${item.value}` },
        ];

        render(<NavigableList {...defaultProps} columns={sortableCols} />);
        await user.click(screen.getByText("Name"));
        expect(onSort).toHaveBeenCalledTimes(1);
    });

    it("makes sortable headers have cursor-pointer class", () => {
        const sortableCols: NavigableListColumn<TestItem>[] = [
            { key: "name", header: "Name", cell: (item) => item.name, onHeaderClick: vi.fn() },
            { key: "value", header: "Value", cell: (item) => `${item.value}` },
        ];
        const { container } = render(<NavigableList {...defaultProps} columns={sortableCols} />);
        const headers = container.querySelectorAll("th");
        expect(headers[0].className).toContain("cursor-pointer");
        expect(headers[1].className).not.toContain("cursor-pointer");
    });

    // ── Chevrons ───────────────────────────────────────────────

    it("renders chevrons when showChevron is true (default)", () => {
        const { container } = render(<NavigableList {...defaultProps} />);
        const chevrons = container.querySelectorAll("svg.lucide-chevron-right");
        expect(chevrons.length).toBe(items.length);
    });

    it("hides chevrons when showChevron is false", () => {
        const { container } = render(<NavigableList {...defaultProps} showChevron={false} />);
        const chevrons = container.querySelectorAll("svg.lucide-chevron-right");
        expect(chevrons.length).toBe(0);
    });

    // ── Custom row IDs and test IDs ────────────────────────────

    it("sets row IDs with custom rowIdPrefix", () => {
        const { container } = render(
            <NavigableList {...defaultProps} rowIdPrefix="custom-row" />
        );
        expect(container.querySelector("#custom-row-0")).toBeDefined();
        expect(container.querySelector("#custom-row-1")).toBeDefined();
    });

    it("sets data-testid with testIdPrefix", () => {
        render(<NavigableList {...defaultProps} testIdPrefix="item" />);
        expect(screen.getByTestId("item-1")).toBeDefined();
        expect(screen.getByTestId("item-2")).toBeDefined();
        expect(screen.getByTestId("item-3")).toBeDefined();
    });

    // ── Header slot ────────────────────────────────────────────

    it("renders headerSlot content above the table", () => {
        render(
            <NavigableList
                {...defaultProps}
                headerSlot={<div data-testid="custom-header">Filters</div>}
            />
        );
        expect(screen.getByTestId("custom-header")).toBeDefined();
        expect(screen.getByText("Filters")).toBeDefined();
    });

    // ── onKeyDownRef ───────────────────────────────────────────

    it("exposes keyboard handler via onKeyDownRef", () => {
 
        const ref = createRef<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>() as React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>;
        ref.current = null;

        render(<NavigableList {...defaultProps} onKeyDownRef={ref} />);
        expect(ref.current).toBeDefined();
        expect(typeof ref.current).toBe("function");
    });
});
