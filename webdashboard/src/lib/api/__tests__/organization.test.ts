import { beforeEach, describe, expect, it, vi } from "vitest";

import { client } from "../client";
import { fetchOrganizationMembers, fetchOrganizationRoles } from "../organization";

vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
  },
}));

describe("organization API", () => {
  beforeEach(() => vi.clearAllMocks());

  it("unwraps paginated members response before returning members", async () => {
    vi.mocked(client.GET).mockResolvedValueOnce({
      data: {
        data: [{ user_id: "user-1", email: "member@example.invalid" }],
        total: 1,
        page: 1,
        limit: 50,
      },
    } as never);

    await expect(fetchOrganizationMembers()).resolves.toEqual([
      { user_id: "user-1", email: "member@example.invalid" },
    ]);
    expect(client.GET).toHaveBeenCalledWith("/api/v1/members");
  });

  it("returns an array for an empty paginated members response", async () => {
    vi.mocked(client.GET).mockResolvedValueOnce({
      data: {
        data: [],
        total: 0,
        page: 1,
        limit: 50,
      },
    } as never);

    const members = await fetchOrganizationMembers();

    expect(Array.isArray(members)).toBe(true);
    expect(members.find((member) => member.user_id === "missing-user")).toBeUndefined();
  });

  it("unwraps paginated roles response before returning roles", async () => {
    vi.mocked(client.GET).mockResolvedValueOnce({
      data: {
        data: [{ id: "role-1", name: "Admin" }],
        total: 1,
        page: 1,
        limit: 50,
      },
    } as never);

    await expect(fetchOrganizationRoles()).resolves.toEqual([
      { id: "role-1", name: "Admin" },
    ]);
    expect(client.GET).toHaveBeenCalledWith("/api/v1/roles");
  });
});
