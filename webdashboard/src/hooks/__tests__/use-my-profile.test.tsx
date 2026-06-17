import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import { useMyProfile, useEmergencyContacts } from "../use-my-profile";

// Mock the API functions
vi.mock("@/lib/api", () => ({
  fetchMyProfile: vi.fn(),
  updateMyProfile: vi.fn(),
  fetchMyEmergencyContacts: vi.fn(),
  addEmergencyContact: vi.fn(),
  deleteEmergencyContact: vi.fn(),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return { Wrapper, queryClient };
};

describe("useMyProfile", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches profile and maps employee & profile data", async () => {
    const mockProfileResponse = {
      employee: { id: "emp-1", full_name: "John Doe" },
      profile: { notes: "my notes" },
      emergency_contacts: [],
    };
    vi.mocked(api.fetchMyProfile).mockResolvedValue(mockProfileResponse as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyProfile(), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.employee).toEqual(mockProfileResponse.employee);
    expect(result.current.profile).toEqual(mockProfileResponse.profile);
    expect(result.current.emergencyContacts).toEqual([]);
  });

  it("mutates profile and calls updateMyProfile", async () => {
    const mockProfile = { id: "p-1", phone: "123" };
    vi.mocked(api.updateMyProfile).mockResolvedValue(mockProfile as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyProfile(), {
      wrapper: Wrapper,
    });

    const updatePromise = result.current.updateProfile({ phone: "123" });
    await expect(updatePromise).resolves.toEqual(mockProfile);
    expect(api.updateMyProfile).toHaveBeenCalledWith({ phone: "123" });
  });
});

describe("useEmergencyContacts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches, adds, and deletes emergency contacts", async () => {
    const mockContacts = [{ id: "c-1", name: "Jane" }];
    vi.mocked(api.fetchMyEmergencyContacts).mockResolvedValue(mockContacts as any);
    vi.mocked(api.addEmergencyContact).mockResolvedValue({ id: "c-2", name: "Bob" } as any);
    vi.mocked(api.deleteEmergencyContact).mockResolvedValue(undefined as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useEmergencyContacts(), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.contacts).toEqual(mockContacts);

    await result.current.addContact({ contact_name: "Bob", relationship: "Friend", phone: "222" });
    expect(api.addEmergencyContact).toHaveBeenCalledWith({ contact_name: "Bob", relationship: "Friend", phone: "222" });

    await result.current.deleteContact("c-1");
    expect(api.deleteEmergencyContact).toHaveBeenCalledWith("c-1");
  });
});
