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
});
