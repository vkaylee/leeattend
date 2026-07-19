import { describe, it, expect, beforeEach, afterEach } from "vitest";

import {
  clearChunkRetry,
  isChunkLoadError,
  shouldRetryChunkLoad,
} from "../chunk-error";

describe("isChunkLoadError", () => {
  it("returns false for undefined", () => {
    expect(isChunkLoadError(undefined)).toBe(false);
  });

  it("matches errors named ChunkLoadError", () => {
    const error = new Error("boom");
    error.name = "ChunkLoadError";
    expect(isChunkLoadError(error)).toBe(true);
  });

  it("matches 'Loading chunk N failed'", () => {
    expect(isChunkLoadError(new Error("Loading chunk 42 failed."))).toBe(true);
  });

  it("matches 'Loading CSS chunk failed'", () => {
    expect(
      isChunkLoadError(new Error("Loading CSS chunk vendor-ui failed.")),
    ).toBe(true);
  });

  it("matches 'Failed to fetch dynamically imported module'", () => {
    expect(
      isChunkLoadError(
        new Error(
          "Failed to fetch dynamically imported module: https://x/roster-a1b2.js",
        ),
      ),
    ).toBe(true);
  });

  it("matches 'error loading dynamically imported module'", () => {
    expect(
      isChunkLoadError(
        new Error("error loading dynamically imported module"),
      ),
    ).toBe(true);
  });

  it("matches Safari's 'Importing a module script failed'", () => {
    expect(
      isChunkLoadError(new Error("Importing a module script failed.")),
    ).toBe(true);
  });

  it("does NOT match an ordinary runtime error", () => {
    expect(isChunkLoadError(new TypeError("x is not a function"))).toBe(false);
  });

  it("does NOT match an error whose stack merely contains .js", () => {
    const error = new Error("Cannot read properties of undefined");
    error.stack = "at foo (https://app/assets/index-abc.js:1:1)";
    expect(isChunkLoadError(error)).toBe(false);
  });
});

describe("shouldRetryChunkLoad", () => {
  beforeEach(() => { sessionStorage.clear(); });
  afterEach(() => { sessionStorage.clear(); });

  it("allows the first 3 reload attempts, then stops", () => {
    expect(shouldRetryChunkLoad()).toBe(true);
    expect(shouldRetryChunkLoad()).toBe(true);
    expect(shouldRetryChunkLoad()).toBe(true);
    // Cap reached — a genuinely broken deploy must not loop forever.
    expect(shouldRetryChunkLoad()).toBe(false);
    expect(shouldRetryChunkLoad()).toBe(false);
  });

  it("persists the count across calls via sessionStorage", () => {
    shouldRetryChunkLoad();
    expect(sessionStorage.getItem("chunkRetry")).toBe("1");
    shouldRetryChunkLoad();
    expect(sessionStorage.getItem("chunkRetry")).toBe("2");
  });
});

describe("clearChunkRetry", () => {
  beforeEach(() => { sessionStorage.clear(); });
  afterEach(() => { sessionStorage.clear(); });

  it("resets the counter so a fresh set of retries is available", () => {
    shouldRetryChunkLoad();
    shouldRetryChunkLoad();
    shouldRetryChunkLoad();
    expect(shouldRetryChunkLoad()).toBe(false);

    // A successful mount clears the counter — the next deploy gets full budget.
    clearChunkRetry();
    expect(shouldRetryChunkLoad()).toBe(true);
  });
});
