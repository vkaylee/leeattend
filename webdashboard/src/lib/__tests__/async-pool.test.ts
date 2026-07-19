import { describe, it, expect } from "vitest";

import { runPool } from "../async-pool";

describe("runPool", () => {
  it("never exceeds the concurrency cap", async () => {
    const items = Array.from({ length: 20 }, (_, i) => i);
    let inFlight = 0;
    let maxInFlight = 0;

    await runPool(items, 4, async () => {
      inFlight += 1;
      maxInFlight = Math.max(maxInFlight, inFlight);
      await new Promise(resolve => setTimeout(resolve, 5));
      inFlight -= 1;
    });

    expect(maxInFlight).toBeLessThanOrEqual(4);
  });

  it("processes every item", async () => {
    const items = [1, 2, 3, 4, 5];
    const seen: number[] = [];

    const failures = await runPool(items, 2, async item => {
      seen.push(item);
    });

    expect(seen.sort()).toEqual(items);
    expect(failures).toHaveLength(0);
  });

  it("settles all items despite failures, reporting each failed item", async () => {
    const items = [1, 2, 3, 4];
    const completed: number[] = [];

    const failures = await runPool(items, 2, async item => {
      if (item % 2 === 0) throw new Error(`fail ${item}`);
      completed.push(item);
    });

    // Odd items succeeded, even items failed — no early abort.
    expect(completed.sort()).toEqual([1, 3]);
    expect(failures.map(f => f.item).sort()).toEqual([2, 4]);
    expect(failures[0].error).toBeInstanceOf(Error);
  });

  it("handles an empty item list", async () => {
    const failures = await runPool([], 4, async () => {
      throw new Error("should not run");
    });
    expect(failures).toHaveLength(0);
  });

  it("coerces a concurrency below 1 up to 1", async () => {
    const items = [1, 2, 3];
    let inFlight = 0;
    let maxInFlight = 0;

    await runPool(items, 0, async () => {
      inFlight += 1;
      maxInFlight = Math.max(maxInFlight, inFlight);
      await new Promise(resolve => setTimeout(resolve, 1));
      inFlight -= 1;
    });

    expect(maxInFlight).toBe(1);
  });

  it("invokes onItemStart before the worker and onItemSettle after, once per item", async () => {
    const items = [10, 20, 30];
    const started: number[] = [];
    const settled: { item: number; error: unknown }[] = [];

    await runPool(
      items,
      2,
      async () => {
        await new Promise(resolve => setTimeout(resolve, 1));
      },
      {
        onItemStart: item => { started.push(item); },
        onItemSettle: (item, _index, error) => { settled.push({ item, error }); },
      },
    );

    expect(started.sort()).toEqual(items);
    expect(settled.map(s => s.item).sort()).toEqual(items);
    expect(settled.every(s => s.error === undefined)).toBe(true);
  });

  it("passes the thrown error to onItemSettle for failed items", async () => {
    const items = [1, 2, 3, 4];
    const settled: { item: number; error: unknown }[] = [];

    await runPool(
      items,
      2,
      async item => {
        if (item % 2 === 0) throw new Error(`fail ${item}`);
      },
      {
        onItemSettle: (item, _index, error) => { settled.push({ item, error }); },
      },
    );

    const failed = settled.filter(s => s.error !== undefined).map(s => s.item);
    const ok = settled.filter(s => s.error === undefined).map(s => s.item);
    expect(failed.sort()).toEqual([2, 4]);
    expect(ok.sort()).toEqual([1, 3]);
  });

  it("reports onItemStart with the item's index", async () => {
    const items = ["a", "b", "c"];
    const byIndex: string[] = [];

    await runPool(
      items,
      1,
      async () => { /* no-op */ },
      {
        onItemStart: (item, index) => { byIndex[index] = item; },
      },
    );

    expect(byIndex).toEqual(items);
  });
});
