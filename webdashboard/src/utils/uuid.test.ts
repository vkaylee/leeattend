import { describe, expect, it, vi } from "vitest";

import { generateUUID } from "./uuid";

describe("generateUUID", () => {
  it("generates a UUID when randomUUID is unavailable", () => {
    vi.stubGlobal("crypto", {
      getRandomValues: (values: Uint8Array) => {
        values.set(new Uint8Array(16).fill(0));
        return values;
      },
    });

    expect(generateUUID()).toBe("00000000-0000-4000-8000-000000000000");
  });
});
