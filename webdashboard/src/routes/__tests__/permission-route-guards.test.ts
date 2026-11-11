import { beforeEach, describe, expect, it, vi } from "vitest";

const { redirect, fetchMyPermissions } = vi.hoisted(() => ({
  redirect: vi.fn((options: { to: string }) => ({ ...options, isRedirect: true })),
  fetchMyPermissions: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
  createFileRoute: () => (options: unknown) => ({ options }),
  redirect,
  Outlet: () => null,
}));

vi.mock("@/lib/api", () => ({ fetchMyPermissions }));
vi.mock("@/components/layout/app-shell", () => ({ default: () => null }));
vi.mock("@/components/common/error-boundary", () => ({
  ErrorBoundary: ({ children }: { children: unknown }) => children,
}));
vi.mock("@/pages/settings/components/settings-sidebar", () => ({
  SettingsSidebar: () => null,
}));
vi.mock("@/components/ui/button", () => ({ default: () => null }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { Route as InfraRoute } from "../_authenticated/_infra";
import { Route as SettingsRoute } from "../_authenticated/_organization/settings";

interface RouteOptions {
  beforeLoad?: () => Promise<void>;
}

const settingsBeforeLoad = (SettingsRoute.options as RouteOptions).beforeLoad!;
const infraBeforeLoad = (InfraRoute.options as RouteOptions).beforeLoad!;

describe("permission route guards", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("allows settings users with any settings permission", async () => {
    fetchMyPermissions.mockResolvedValue({
      permissions: ["organization:settings:read"],
    });

    await expect(settingsBeforeLoad()).resolves.toBeUndefined();
    expect(redirect).not.toHaveBeenCalled();
  });

  it("redirects users without settings permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["ess:attendance"] });

    await expect(settingsBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it("redirects users without infrastructure permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["employees:read"] });

    await expect(infraBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it("fails closed when permission lookup fails", async () => {
    fetchMyPermissions.mockRejectedValue(new Error("permission service unavailable"));

    await expect(infraBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });
});
