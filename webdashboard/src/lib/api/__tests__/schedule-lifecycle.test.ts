import { beforeEach, describe, expect, it, vi } from "vitest";

import { apiFetch } from "../helpers";
import {
  commitScheduleReplan,
  fetchScheduleEvidence,
  fetchScheduleHistory,
  previewScheduleReplan,
  previewSchedules,
  publishSchedules,
} from "../schedule-lifecycle";

vi.mock("../helpers", () => ({ apiFetch: vi.fn() }));

function response(body: unknown): Response {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" } });
}

const replacement = {
  status: "work" as const,
  source: "manual",
  shift_version_id: "shift-version-1",
  intervals: [{ sort_order: 0, start: "08:00:00", end: "17:00:00", breaks: [] }],
};

beforeEach(() => vi.clearAllMocks());

describe("schedule lifecycle API", () => {
  it("sends provisional preview request with exact route", async () => {
    vi.mocked(apiFetch).mockResolvedValue(response({ provisional: true }));
    await previewSchedules({
      employment_ids: ["employment-1"],
      start_date: "2026-11-01",
      end_date: "2026-11-01",
      timezone: "UTC",
      mode: "office",
      approval_instance_id: null,
      reason: "Publish weekly roster",
      retroactive: false,
      expected_revisions: { "employment-1:2026-11-01": 0 },
    });
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toBe("/api/v1/schedules/preview");
  });

  it("adds idempotency keys to publish and replan commits", async () => {
    vi.mocked(apiFetch).mockResolvedValue(response({}));
    await publishSchedules({
      employment_ids: ["employment-1"],
      start_date: "2026-11-01",
      end_date: "2026-11-01",
      timezone: "UTC",
      approval_instance_id: null,
      reason: "Publish",
      retroactive: false,
      expected_revisions: { "employment-1:2026-11-01": 0 },
      preview_id: "preview-1",
      preview_hash: "a".repeat(64),
    }, "publish-key");
    await commitScheduleReplan("employment-1", "2026-11-01", {
      expected_revision: 1,
      reason: "Correction",
      replacement,
      preview_id: "preview-2",
      preview_hash: "b".repeat(64),
      confirm_diff_hash: "c".repeat(64),
    }, "replan-key");
    expect((vi.mocked(apiFetch).mock.calls[0][1] as RequestInit).headers).toMatchObject({ "Idempotency-Key": "publish-key" });
    expect((vi.mocked(apiFetch).mock.calls[1][1] as RequestInit).headers).toMatchObject({ "Idempotency-Key": "replan-key" });
  });

  it("uses history and evidence routes with encoded path values", async () => {
    vi.mocked(apiFetch).mockResolvedValue(response({ data: [] }));
    await fetchScheduleHistory({ employment_id: "employment-1", limit: 50 });
    await fetchScheduleEvidence("employment/1", "2026-11-01");
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/api/v1/schedules/history");
    expect(vi.mocked(apiFetch).mock.calls[1][0]).toBe("/api/v1/schedules/evidence/employment%2F1/2026-11-01");
  });

  it("sends exact server diff token during replan preview", async () => {
    vi.mocked(apiFetch).mockResolvedValue(response({ diff_hash: "d".repeat(64) }));
    await previewScheduleReplan("employment-1", "2026-11-01", { expected_revision: 1, reason: "Fix", replacement });
    expect(JSON.parse((vi.mocked(apiFetch).mock.calls[0][1] as RequestInit).body as string)).toMatchObject({ expected_revision: 1, replacement });
  });
});
