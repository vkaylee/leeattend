import { describe, it, expect, vi } from "vitest";

import { retryImport, lazyRetry } from "../chunk-loader";

describe("retryImport", () => {
  it("resolves on first successful import", async () => {
    const importer = vi.fn().mockResolvedValue("ok");
    const result = await retryImport(importer, 3, 1);
    expect(result).toBe("ok");
    expect(importer).toHaveBeenCalledTimes(1);
  });

  it("retries on failure and eventually succeeds", async () => {
    const importer = vi
      .fn()
      .mockRejectedValueOnce(new Error("fail1"))
      .mockRejectedValueOnce(new Error("fail2"))
      .mockRejectedValueOnce(new Error("fail3"))
      .mockResolvedValue("recovered");

    const result = await retryImport(importer, 3, 1);

    expect(result).toBe("recovered");
    expect(importer).toHaveBeenCalledTimes(4); // initial + 3 retries
  });

  it("rejects after exhausting all retries", async () => {
    const importer = vi.fn().mockRejectedValue(new Error("permanent"));

    await expect(retryImport(importer, 2, 1)).rejects.toThrow("permanent");
    expect(importer).toHaveBeenCalledTimes(3); // initial + 2 retries
  });

  it("defaults to 3 retries when not specified", async () => {
    const importer = vi.fn().mockRejectedValue(new Error("fail"));

    await expect(retryImport(importer)).rejects.toThrow("fail");
    expect(importer).toHaveBeenCalledTimes(4); // initial + 3 retries
  });
});

describe("lazyRetry", () => {
  it("wraps importer function", async () => {
    const importer = vi.fn().mockResolvedValue({ default: "Component" });
    const wrapped = lazyRetry(importer, 2);

    const result = await wrapped();
    expect(result).toEqual({ default: "Component" });
    expect(importer).toHaveBeenCalledTimes(1);
  });

  it("retries on failure", async () => {
    const importer = vi
      .fn()
      .mockRejectedValueOnce(new Error("fail1"))
      .mockRejectedValueOnce(new Error("fail2"))
      .mockResolvedValue({ default: "Retried" });

    const result = await lazyRetry(importer, 2)();
    expect(result).toEqual({ default: "Retried" });
    expect(importer).toHaveBeenCalledTimes(3); // initial + 2 retries
  });

  it("rejects after max retries", async () => {
    const importer = vi.fn().mockRejectedValue(new Error("maxed"));

    await expect(lazyRetry(importer, 1)()).rejects.toThrow("maxed");
    expect(importer).toHaveBeenCalledTimes(2); // initial + 1 retry
  });
});
