import { describe, expect, it } from "vitest";

import { departmentKeys, employeeKeys } from "@/lib/query-keys";

describe("single-organization query-key factories", () => {
  it("keeps department descendants under the organization-independent prefix", () => {
    const all = departmentKeys.all();

    expect(departmentKeys.list({ search: "eng" }).slice(0, all.length)).toEqual(all);
    expect(departmentKeys.detail("department-1").slice(0, all.length)).toEqual(all);
    expect(departmentKeys.lookup({ managerId: "employee-1" }).slice(0, all.length)).toEqual(all);
    expect(all).toEqual(["departments"]);
  });

  it("keeps employee descendants under the organization-independent prefix", () => {
    const all = employeeKeys.all();

    expect(employeeKeys.list({ status: "active" }).slice(0, all.length)).toEqual(all);
    expect(employeeKeys.search({ search: "lee" }).slice(0, all.length)).toEqual(all);
    expect(employeeKeys.detail("employee-1").slice(0, all.length)).toEqual(all);
    expect(employeeKeys.lookup({ permission: "picker" }).slice(0, all.length)).toEqual(all);
    expect(all).toEqual(["employees"]);
  });

  it("normalizes equivalent employee filter objects to one key", () => {
    expect(employeeKeys.search({
      customFieldFilters: { department: "engineering", level: "senior" },
      departmentIds: ["department-b", "department-a"],
      search: "lee",
    })).toEqual(employeeKeys.search({
      customFieldFilters: { level: "senior", department: "engineering" },
      departmentIds: ["department-a", "department-b"],
      search: "lee",
    }));
  });

  it("omits undefined filter values", () => {
    expect(departmentKeys.list({ search: undefined })).toEqual(departmentKeys.list());
  });
});
