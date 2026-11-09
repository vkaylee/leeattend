import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock client and helpers before importing the test file
vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
    PUT: vi.fn(),
    POST: vi.fn(),
    DELETE: vi.fn(),
  },
}));


import { claimOt, fetchAttendanceSummaries } from "../attendance";
import { client } from "../client";
import {
  fetchMyProfile,
  updateMyProfile,
  fetchMyEmergencyContacts,
  addEmergencyContact,
  deleteEmergencyContact,
  fetchMyDocuments,
  fetchMyPayslips,
  fetchMyAttendance,
  fetchMyTimeEntries,
  punchMySelf,
  fetchEssLeaveBalances,
  fetchEssLeaveRequests,
  fetchMyAttendanceSummaries,
  submitMyOtClaim,
} from "../ess";
describe("Employee Self-Service (ESS) API Client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetchMyProfile should make a GET call to /v1/ess/profile", async () => {
    const mockProfile = { employee: { id: "emp-1" }, profile: { notes: "test" } };
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockProfile } as any);

    const result = await fetchMyProfile();
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/profile");
    expect(result).toEqual(mockProfile);
  });

  it("fetchMyProfile returns null when no data is returned (no longer throws)", async () => {
    vi.mocked(client.GET).mockResolvedValueOnce({ data: null } as any);
    const result = await fetchMyProfile();
    expect(result).toBeNull();
  });

  it("updateMyProfile should make a PUT call to /ess/profile", async () => {
    const updateRequest = { phone: "12345" };
    const updatedProfile = { id: "profile-1", phone: "12345" };
    vi.mocked(client.PUT).mockResolvedValueOnce({ data: updatedProfile } as any);

    const result = await updateMyProfile(updateRequest);
    expect(client.PUT).toHaveBeenCalledWith("/v1/ess/profile", { body: updateRequest });
    expect(result).toEqual(updatedProfile);
  });

  it("fetchMyEmergencyContacts should make a GET call", async () => {
    const mockContacts = [{ id: "c-1", name: "Contact 1" }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockContacts } as any);

    const result = await fetchMyEmergencyContacts();
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/emergency-contacts");
    expect(result).toEqual(mockContacts);
  });

  it("addEmergencyContact should make a POST call", async () => {
    const newContact = { contact_name: "Contact 2", relationship: "Spouse", phone: "111" };
    vi.mocked(client.POST).mockResolvedValueOnce({ data: newContact } as any);

    const result = await addEmergencyContact(newContact);
    expect(client.POST).toHaveBeenCalledWith("/v1/ess/emergency-contacts", { body: newContact });
    expect(result).toEqual(newContact);
  });

  it("deleteEmergencyContact should make a DELETE call with params", async () => {
    vi.mocked(client.DELETE).mockResolvedValueOnce({} as any);

    await deleteEmergencyContact("contact-uuid");
    expect(client.DELETE).toHaveBeenCalledWith("/v1/ess/emergency-contacts/{contact_id}", {
      params: { path: { contact_id: "contact-uuid" } }
    });
  });

  it("fetchMyDocuments should make a GET call", async () => {
    const mockDocs = [{ id: "doc-1", name: "Doc" }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockDocs } as any);

    const result = await fetchMyDocuments();
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/documents");
    expect(result).toEqual(mockDocs);
  });

  it("fetchMyPayslips should make a GET call", async () => {
    const mockPayslips = [{ id: "ps-1", net_pay_amount: "100" }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockPayslips } as any);

    const result = await fetchMyPayslips();
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/payslips");
    expect(result).toEqual(mockPayslips);
  });

  it("fetchMyAttendance should make a GET call with query params", async () => {
    const mockAttendance = [{ id: "att-1" }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockAttendance } as any);

    const result = await fetchMyAttendance({ start_date: "2026-03-01", end_date: "2026-03-07" });
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/attendance", {
      params: { query: { start_date: "2026-03-01", end_date: "2026-03-07" } }
    });
    expect(result).toEqual(mockAttendance);
  });

  it("fetchMyTimeEntries should make a GET call with query params", async () => {
    const mockEntries = [{ id: "entry-1" }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockEntries } as any);

    const result = await fetchMyTimeEntries({ start_date: "2026-03-01", end_date: "2026-03-07" });
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/time-entries", {
      params: { query: { start_date: "2026-03-01", end_date: "2026-03-07" } }
    });
    expect(result).toEqual(mockEntries);
  });

  it("punchMySelf should make a POST call with body", async () => {
    const mockPunch = { id: "punch-1" };
    vi.mocked(client.POST).mockResolvedValueOnce({ data: mockPunch } as any);

    const result = await punchMySelf({ note: "punching in" });
    expect(client.POST).toHaveBeenCalledWith("/v1/ess/punch", { body: { note: "punching in" } });
    expect(result).toEqual(mockPunch);
  });

  it("fetchEssLeaveBalances should make a GET call", async () => {
    const mockBalances = [{ leave_type_id: "lt-1", balance_days: 10 }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockBalances } as any);

    const result = await fetchEssLeaveBalances();
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/leave/balances");
    expect(result).toEqual(mockBalances);
  });

  it("fetchEssLeaveRequests should make a GET call", async () => {
    const mockRequests = [{ id: "req-1" }];
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockRequests } as any);

    const result = await fetchEssLeaveRequests();
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/leave/requests");
    expect(result).toEqual(mockRequests);
  });

  it("fetchMyAttendanceSummaries should fetch via fetchAttendanceSummaries helper", async () => {
    const result = await fetchMyAttendanceSummaries("2026-03-01", "2026-03-07");
    expect(fetchAttendanceSummaries).toHaveBeenCalledWith("2026-03-01", "2026-03-07");
    expect(result).toEqual([{ date: "2026-03-01", status: "Present" }]);
  });

  it("submitMyOtClaim should fetch profile and submit OT claim", async () => {
    const mockProfile = { employee: { id: "emp-123" } };
    vi.mocked(client.GET).mockResolvedValueOnce({ data: mockProfile } as never);

    await submitMyOtClaim("2026-03-01", 120, "OT work");
    expect(client.GET).toHaveBeenCalledWith("/v1/ess/profile");
    expect(claimOt).toHaveBeenCalledWith("emp-123", {
      date: "2026-03-01",
      claimed_minutes: 120,
      reason: "OT work"
    });
  });
});
