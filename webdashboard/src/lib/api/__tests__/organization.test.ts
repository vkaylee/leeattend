import { beforeEach, describe, expect, it, vi } from "vitest";

import { client } from "../client";
import { fetchOrganizationRoles } from "../organization";

vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
  },
}));

describe("organization API", () => {
  beforeEach(() => vi.clearAllMocks());

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
