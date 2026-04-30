import { describe, it, expect, vi, beforeEach } from "vitest";

const mockCallbacks: string[] = [];

vi.mock("web-vitals", () => ({
 
  onCLS: vi.fn((_cb: (metric: unknown) => void) => mockCallbacks.push("onCLS")),
 
  onFCP: vi.fn((_cb: (metric: unknown) => void) => mockCallbacks.push("onFCP")),
 
  onINP: vi.fn((_cb: (metric: unknown) => void) => mockCallbacks.push("onINP")),
 
  onLCP: vi.fn((_cb: (metric: unknown) => void) => mockCallbacks.push("onLCP")),
 
  onTTFB: vi.fn((_cb: (metric: unknown) => void) => mockCallbacks.push("onTTFB")),
}));

describe("web-vitals", () => {
  beforeEach(() => {
    mockCallbacks.length = 0;
  });

  it("trackWebVitals registers all five metric observers", async () => {
    const { trackWebVitals } = await import("../web-vitals");
    trackWebVitals();

    expect(mockCallbacks).toContain("onCLS");
    expect(mockCallbacks).toContain("onFCP");
    expect(mockCallbacks).toContain("onINP");
    expect(mockCallbacks).toContain("onLCP");
    expect(mockCallbacks).toContain("onTTFB");
  });

  it("reports metrics in DEV mode without throwing", async () => {
    const { trackWebVitals } = await import("../web-vitals");
    expect(() => trackWebVitals()).not.toThrow();
  });
});
