import { beforeEach, describe, expect, it, vi } from "vitest";

const { redirect, clearAuthSession } = vi.hoisted(() => ({
  redirect: vi.fn((options: { to: string }) => ({ ...options, isRedirect: true })),
  clearAuthSession: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
  createFileRoute: () => (options: unknown) => ({ options }),
  redirect,
  Outlet: () => null,
}));

vi.mock("@/components/ui/typography", () => ({
  Typography: () => null,
}));

vi.mock("@/hooks/use-profile", () => ({
  profileKeys: { me: ["profile-me"] },
}));

vi.mock("@/lib/api", () => ({
  fetchMe: vi.fn(),
  clearAuthSession,
}));

import { Route } from "../_authenticated";

type BeforeLoad = (args: {
  context: {
    queryClient: {
      ensureQueryData: (options: { queryFn: () => Promise<unknown> }) => Promise<unknown>;
    };
  };
}) => Promise<unknown>;

describe("authenticated route guard behavior", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("keeps an authenticated user after profile bootstrap", async () => {
    const user = { id: "user-1" };
    const beforeLoad = Route.options.beforeLoad as BeforeLoad;

    await expect(beforeLoad({
      context: {
        queryClient: {
          ensureQueryData: vi.fn().mockResolvedValue(user),
        },
      },
    })).resolves.toEqual({ user });
    expect(clearAuthSession).not.toHaveBeenCalled();
    expect(redirect).not.toHaveBeenCalled();
  });

  it("redirects when profile bootstrap returns no user", async () => {
    const beforeLoad = Route.options.beforeLoad as BeforeLoad;

    await expect(beforeLoad({
      context: {
        queryClient: {
          ensureQueryData: vi.fn().mockResolvedValue(null),
        },
      },
    })).rejects.toMatchObject({ to: "/login", isRedirect: true });
    expect(clearAuthSession).toHaveBeenCalled();
    expect(redirect).toHaveBeenCalledWith({ to: "/login" });
  });
});
