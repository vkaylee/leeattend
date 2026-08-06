import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import ResponsiveTable from "../responsive-table";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

interface TestRow {
  id: string;
  name: string;
}

const rows: TestRow[] = [
  { id: "1", name: "Alpha" },
  { id: "2", name: "Beta" },
];

const columns = [
  {
    accessorKey: "name",
    header: "Name",
  },
];

interface WideRow {
  code: string;
  dept: string;
  id: string;
  name: string;
  sel: string;
}

const wideRows: WideRow[] = [{ id: "1", sel: "x", code: "E-001", name: "Alpha", dept: "Ops" }];

/** Explicit sizes let TanStack compute cumulative pin offsets via column.getStart(). */
const wideColumns = [
  { accessorKey: "sel", header: "Sel", size: 60 },
  { accessorKey: "code", header: "Code", size: 140 },
  { accessorKey: "name", header: "Name", size: 200 },
  { accessorKey: "dept", header: "Dept", size: 220 },
];

/** Desktop-only scope — the mobile card list renders the same rows. */
function desktopOf(container: HTMLElement): Element | null {
  return container.querySelector(".hidden.lg\\:block");
}

describe("ResponsiveTable", () => {
  it("renders SmartTable and mobile cards from one data source", () => {
    const { container } = render(
      <ResponsiveTable<TestRow>
        columns={columns}
        data={rows}
        getCardKey={(row) => row.id}
        getRowId={(row) => row.id}
        renderCard={(row) => <article>{row.name} mobile</article>}
      />,
    );

    expect(screen.getByText("Alpha")).toBeInTheDocument();
    expect(screen.getByText("Alpha mobile")).toBeInTheDocument();
    expect(container.querySelector(".hidden.lg\\:block table")).toBeInTheDocument();
    expect(container.querySelector(".lg\\:hidden")).toBeInTheDocument();
  });

  it("supports a custom desktop renderer", () => {
    render(
      <ResponsiveTable<TestRow>
        data={rows}
        renderCard={(row) => <article>{row.name} card</article>}
        renderDesktopTable={(data) => <div>{data.length} desktop rows</div>}
      />,
    );

    expect(screen.getByText("2 desktop rows")).toBeInTheDocument();
    expect(screen.getByText("Beta card")).toBeInTheDocument();
  });

  it("renders a translated empty state on mobile and desktop", () => {
    render(
      <ResponsiveTable<TestRow>
        columns={columns}
        data={[]}
        emptyState={{ icon: <span>empty icon</span>, title: "Nothing here" }}
        getRowId={(row) => row.id}
        renderCard={(row) => <article>{row.name}</article>}
      />,
    );

    expect(screen.getAllByText("Nothing here")).toHaveLength(2);
    expect(screen.getByText("empty icon")).toBeInTheDocument();
  });

  it("keeps existing cards visible while additional data loads", () => {
    render(
      <ResponsiveTable<TestRow>
        columns={columns}
        data={rows}
        getRowId={(row) => row.id}
        isLoading
        renderCard={(row) => <article>{row.name} retained</article>}
      />,
    );

    expect(screen.getByText("Alpha retained")).toBeInTheDocument();
    expect(screen.getByText("Beta retained")).toBeInTheDocument();
  });

  describe("column pinning", () => {
    it("pins no columns by default", () => {
      const { container } = render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
        />,
      );

      const headers = desktopOf(container)?.querySelectorAll("th") ?? [];
      expect(headers.length).toBe(4);
      headers.forEach((th) => {
        expect((th as HTMLElement).style.position).not.toBe("sticky");
      });
    });

    it("forwards stickyLeftColumns and pins that many leading columns", () => {
      const { container } = render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
          stickyLeftColumns={2}
        />,
      );

      const headers = Array.from(desktopOf(container)?.querySelectorAll("th") ?? []) as HTMLElement[];
      expect(headers[0].style.position).toBe("sticky");
      expect(headers[1].style.position).toBe("sticky");
      expect(headers[2].style.position).not.toBe("sticky");
      expect(headers[3].style.position).not.toBe("sticky");
    });

    /**
     * The offsets are the point of the prop: column N sits at the cumulative width
     * of the columns pinned before it. Hand-written sticky tables hardcoded these
     * as Tailwind classes and silently broke when a width changed.
     */
    it("stacks pinned columns at cumulative left offsets", () => {
      const { container } = render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
          stickyLeftColumns={3}
        />,
      );

      const headers = Array.from(desktopOf(container)?.querySelectorAll("th") ?? []) as HTMLElement[];
      expect(headers[0].style.left).toBe("0px");
      expect(headers[1].style.left).toBe("60px");
      expect(headers[2].style.left).toBe("200px");
    });

    it("applies the same offsets to body cells so columns stay aligned", () => {
      const { container } = render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
          stickyLeftColumns={2}
        />,
      );

      const cells = Array.from(
        desktopOf(container)?.querySelectorAll("tbody td") ?? [],
      ) as HTMLElement[];
      expect(cells[0].style.position).toBe("sticky");
      expect(cells[0].style.left).toBe("0px");
      expect(cells[1].style.position).toBe("sticky");
      expect(cells[1].style.left).toBe("60px");
      expect(cells[2].style.position).not.toBe("sticky");
    });

    it("pins trailing columns via stickyRightColumns", () => {
      const { container } = render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
          stickyRightColumns={1}
        />,
      );

      const headers = Array.from(desktopOf(container)?.querySelectorAll("th") ?? []) as HTMLElement[];
      expect(headers[3].style.position).toBe("sticky");
      expect(headers[3].style.right).toBe("0px");
      expect(headers[0].style.position).not.toBe("sticky");
    });
  });

  describe("expanded rows", () => {
    it("renders a detail row spanning every visible column", () => {
      const { container } = render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
          renderExpandedRow={(row) => <div>detail for {row.name}</div>}
        />,
      );

      const detail = screen.getByText(/detail for Alpha/);
      const cell = detail.closest("td");
      expect(cell?.getAttribute("colSpan")).toBe("4");
      expect(desktopOf(container)?.contains(detail)).toBe(true);
    });

    it("omits the detail row when the renderer returns nothing", () => {
      render(
        <ResponsiveTable<WideRow>
          columns={wideColumns}
          data={wideRows}
          getRowId={(row) => row.id}
          renderCard={(row) => <article>{row.name}</article>}
          renderExpandedRow={() => null}
        />,
      );

      expect(screen.queryByText(/detail for/)).not.toBeInTheDocument();
    });
  });

  it("applies a per-row className from getRowClassName", () => {
    const { container } = render(
      <ResponsiveTable<WideRow>
        columns={wideColumns}
        data={wideRows}
        getRowClassName={(row) => `marked-${row.code}`}
        getRowId={(row) => row.id}
        renderCard={(row) => <article>{row.name}</article>}
      />,
    );

    const row = desktopOf(container)?.querySelector("tbody tr");
    expect(row?.className).toContain("marked-E-001");
  });

  it("renders custom skeleton rows while loading", () => {
    render(
      <ResponsiveTable<WideRow>
        columns={wideColumns}
        data={[]}
        getRowId={(row) => row.id}
        isLoading
        renderCard={(row) => <article>{row.name}</article>}
        renderSkeletonRow={(i) => (
          <tr>
            <td>skeleton row {i}</td>
          </tr>
        )}
        skeletonCount={3}
      />,
    );

    expect(screen.getByText("skeleton row 0")).toBeInTheDocument();
    expect(screen.getByText("skeleton row 2")).toBeInTheDocument();
    expect(screen.queryByText("Loading...")).not.toBeInTheDocument();
  });
});
