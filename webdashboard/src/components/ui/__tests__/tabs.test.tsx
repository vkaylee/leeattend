import { fireEvent, render, screen } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "../tabs";

describe("Tabs", () => {
  it("exposes tab and panel linkage and selects with keyboard navigation", () => {
    const onValueChange = vi.fn();
    render(
      <Tabs onValueChange={onValueChange} value="one">
        <TabsList aria-label="Sections">
          <TabsTrigger value="one">One</TabsTrigger>
          <TabsTrigger value="two">Two</TabsTrigger>
          <TabsContent value="one">One content</TabsContent>
        </TabsList>
      </Tabs>
    );

    const first = screen.getByRole("tab", { name: "One" });
    const second = screen.getByRole("tab", { name: "Two" });
    const panel = screen.getByRole("tabpanel");
    expect(first).toHaveAttribute("aria-controls", panel.id);
    expect(panel).toHaveAttribute("aria-labelledby", first.id);

    first.focus();
    fireEvent.keyDown(first, { key: "ArrowRight" });
    expect(second).toHaveFocus();
    expect(onValueChange).toHaveBeenCalledWith("two");
  });

  it("skips disabled tabs and supports Home and End", () => {
    const onValueChange = vi.fn();
    render(
      <Tabs onValueChange={onValueChange} value="one">
        <TabsList>
          <TabsTrigger value="one">One</TabsTrigger>
          <TabsTrigger disabled value="two">Two</TabsTrigger>
          <TabsTrigger value="three">Three</TabsTrigger>
        </TabsList>
      </Tabs>
    );

    const first = screen.getByRole("tab", { name: "One" });
    const last = screen.getByRole("tab", { name: "Three" });
    fireEvent.keyDown(first, { key: "ArrowRight" });
    expect(last).toHaveFocus();
    fireEvent.keyDown(last, { key: "Home" });
    expect(first).toHaveFocus();
    fireEvent.keyDown(first, { key: "End" });
    expect(last).toHaveFocus();
  });

  it("composes user handlers without changing selection", () => {
    const onValueChange = vi.fn();
    const onClick = vi.fn();
    render(
      <Tabs onValueChange={onValueChange} value="one">
        <TabsList>
          <TabsTrigger onClick={onClick} value="one">One</TabsTrigger>
        </TabsList>
      </Tabs>
    );

    fireEvent.click(screen.getByRole("tab", { name: "One" }));
    expect(onClick).toHaveBeenCalledOnce();
    expect(onValueChange).toHaveBeenCalledWith("one");
  });
});
