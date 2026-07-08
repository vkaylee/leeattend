import { describe, expect, it } from "vitest";

import { maskEmail, maskPhone } from "./utils";

describe("maskEmail (PII protection)", () => {
  it("masks standard email", () => {
    // 8-char local part → capped at 4 asterisks
    expect(maskEmail("john.doe@example.com")).toBe("j****e@example.com");
  });

  it("masks short local-part email", () => {
    expect(maskEmail("ab@example.com")).toBe("a***b@example.com");
  });

  it("masks single-char local-part email", () => {
    expect(maskEmail("a@example.com")).toBe("a***@example.com");
  });

  it("returns *** for null/undefined/empty", () => {
    expect(maskEmail(null)).toBe("***");
    expect(maskEmail(undefined)).toBe("***");
    expect(maskEmail("")).toBe("***");
  });

  it("returns *** when no @ present", () => {
    expect(maskEmail("not-an-email")).toBe("***");
  });

  it("caps asterisks at 4 for long local-part", () => {
    expect(maskEmail("abcdefghijk@example.com")).toBe("a****k@example.com");
  });
});

describe("maskPhone (PII protection)", () => {
  it("masks local phone number", () => {
    expect(maskPhone("0901234567")).toBe("***4567");
  });

  it("masks international phone number", () => {
    expect(maskPhone("+84901234567")).toBe("***4567");
  });

  it("masks phone with formatting chars", () => {
    expect(maskPhone("+84 90 123 4567")).toBe("***4567");
    expect(maskPhone("090-123-4567")).toBe("***4567");
  });

  it("returns *** for null/undefined/short", () => {
    expect(maskPhone(null)).toBe("***");
    expect(maskPhone(undefined)).toBe("***");
    expect(maskPhone("")).toBe("***");
    expect(maskPhone("123")).toBe("***");
  });
});
