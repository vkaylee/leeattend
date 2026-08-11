import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { PageContainer } from "./page-container";

describe("PageContainer", () => {
  it("provides the shared responsive page rhythm without horizontal padding", () => {
    const { container } = render(
      <PageContainer data-testid="page-container">
        <div>Content</div>
      </PageContainer>,
    );

    const page = container.firstElementChild;
    expect(page).toHaveClass("w-full", "space-y-6", "pb-20");
    expect(page).not.toHaveClass("px-4");
    expect(page).not.toHaveClass("px-6");
    expect(page).not.toHaveClass("px-8");
  });
});
