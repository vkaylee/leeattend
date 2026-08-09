import { describe, expect, it } from "vitest";

import { departmentKeys, employeeKeys } from "@/lib/query-keys";

describe("workspace query-key factories", () => {
  it("keeps department descendants under the workspace all prefix", () => {
    const all = departmentKeys.all("workspace-a");

    expect(departmentKeys.list("workspace-a", { search: "eng" }).slice(0, all.length)).toEqual(all);
    expect(departmentKeys.detail("workspace-a", "department-1").slice(0, all.length)).toEqual(all);
    expect(departmentKeys.lookup("workspace-a", { managerId: "employee-1" }).slice(0, all.length)).toEqual(all);
  });

  it("keeps employee descendants under the workspace all prefix", () => {
    const all = employeeKeys.all("workspace-a");

    expect(employeeKeys.list("workspace-a", { status: "active" }).slice(0, all.length)).toEqual(all);
    expect(employeeKeys.search("workspace-a", { search: "lee" }).slice(0, all.length)).toEqual(all);
    expect(employeeKeys.detail("workspace-a", "employee-1").slice(0, all.length)).toEqual(all);
    expect(employeeKeys.lookup("workspace-a", { permission: "picker" }).slice(0, all.length)).toEqual(all);
  });

  it("separates workspaces at the second key segment", () => {
    expect(employeeKeys.all("workspace-a")).not.toEqual(employeeKeys.all("workspace-b"));
    expect(departmentKeys.all("workspace-a")).not.toEqual(departmentKeys.all("workspace-b"));
  });

  it("normalizes equivalent employee filter objects to one key", () => {
    expect(
      employeeKeys.search("workspace-a", {
        customFieldFilters: { department: "engineering", level: "senior" },
        departmentIds: ["department-b", "department-a"],
        search: "lee",
      }),
    ).toEqual(
      employeeKeys.search("workspace-a", {
        customFieldFilters: { level: "senior", department: "engineering" },
        departmentIds: ["department-a", "department-b"],
        search: "lee",
      }),
    );
  });

  it("omits undefined filter values", () => {
    expect(departmentKeys.list("workspace-a", { search: undefined })).toEqual(
      departmentKeys.list("workspace-a"),
    );
  });
});
