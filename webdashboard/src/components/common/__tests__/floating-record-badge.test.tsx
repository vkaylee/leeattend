import { render } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import FloatingRecordBadge from "../floating-record-badge";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "common.showing": "showing",
        "common.records": "records",
      };
      return translations[key] || key;
    },
  }),
}));

describe("FloatingRecordBadge", () => {
    it("renders loaded and total counts", () => {
        const { container } = render(<FloatingRecordBadge loaded={5} total={20} />);
        expect(container.textContent).toContain("5");
        expect(container.textContent).toContain("20");
    });

    it("renders custom label", () => {
        const { container } = render(
            <FloatingRecordBadge label="items" loaded={3} total={10} />
        );
        expect(container.textContent).toContain("items");
    });

    it("renders default 'records' label", () => {
        const { container } = render(
            <FloatingRecordBadge loaded={1} total={1} />
        );
        expect(container.textContent).toContain("records");
    });

    it("uses fixed bottom positioning with mobile offset for bottom nav", () => {
        const { container } = render(<FloatingRecordBadge loaded={1} total={1} />);
        const wrapper = container.firstChild as HTMLElement;
        expect(wrapper.className).toContain("fixed");
        expect(wrapper.className).toContain("bottom-22");
        expect(wrapper.className).toContain("md:bottom-6");
    });

    it("renders default variant with slate/primary colors", () => {
        const { container } = render(<FloatingRecordBadge loaded={1} total={1} />);
        const badge = container.querySelector(".border-slate-200");
        expect(badge).toBeDefined();
    });

    it("renders amber variant with amber colors", () => {
        const { container } = render(
            <FloatingRecordBadge loaded={1} total={1} variant="amber" />
        );
        const badge = container.querySelector(".border-amber-200");
        expect(badge).toBeDefined();
    });
});
