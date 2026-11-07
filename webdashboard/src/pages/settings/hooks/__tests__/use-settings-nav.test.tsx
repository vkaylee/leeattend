import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useSettingsNav } from "../use-settings-nav";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: vi.fn(() => true),
  }),
}));

describe("useSettingsNav", () => {
  it("does not expose Audit Logs in Settings Hub navigation", () => {
    const { result } = renderHook(() => useSettingsNav());

    const systemPaths = result.current.systemGroup.map((item) => item.path);
    const operationsPaths = result.current.operationsGroup.map((item) => item.path);
    const accessPaths = result.current.accessGroup.map((item) => item.path);

    expect(systemPaths).not.toContain("/audit-logs");
    expect(systemPaths).toContain("/workspace/settings/storage");
    expect(systemPaths).toContain("/workspace/settings/smtp");
    expect(operationsPaths).toContain("/shifts/rules");
    expect(operationsPaths).not.toContain("/workspace/settings/holidays");
    expect(accessPaths).toContain("/workspace/settings/members");
    expect(accessPaths).not.toContain("/workspace/people");
  });
});
