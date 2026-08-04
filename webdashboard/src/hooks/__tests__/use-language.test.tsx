import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

// Hoisted mocks — persist across vi.resetModules() inside the hook
const mockChangeLanguage = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const mockOn = vi.hoisted(() => vi.fn());
const mockOff = vi.hoisted(() => vi.fn());
const mockUpdateMe = vi.hoisted(() => vi.fn().mockResolvedValue({ preferred_language: "vi" }));
const mockLogError = vi.hoisted(() => vi.fn());

// Mock i18next with hoisted mocks
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      language: "vi",
      changeLanguage: mockChangeLanguage,
      on: mockOn,
      off: mockOff,
    },
  }),
}));

// Mock use-profile (test-friendly implementation that satisfies useLanguage)
vi.mock("../use-profile", () => ({
  profileKeys: { me: ["profile-me"] },
  useProfile: () => ({
    user: { id: "1", preferred_language: "vi" },
  }),
}));

vi.mock("@/lib/api", () => ({
  updateMe: (data: unknown) => mockUpdateMe(data),
}));

// Mock logger — the real implementation calls logError on API failure,
// not console.warn
vi.mock("@/lib/logger", () => ({
  logError: mockLogError,
}));

// Mock use-language giữ real implementation nhưng mock use-profile phía trên
// (importOriginal giữ nguyên hook, nó sẽ dùng use-profile mock từ trên)
vi.mock("../use-language", async (importOriginal) => {
  const actual = await importOriginal() as any;
  return {
    ...actual,
    // Wrap useLanguage to test the REAL implementation
    useLanguage: () => actual.useLanguage(),
  };
});

// Import REAL hook
import { useLanguage, SUPPORTED_LANGUAGES } from "../use-language";

const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, "localStorage", { value: localStorageMock });

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

describe("useLanguage (real implementation with mocked deps)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.getItem.mockReturnValue(null);
    // Reset module so hook picks up fresh mock state
    mockChangeLanguage.mockResolvedValue(undefined);
    mockUpdateMe.mockResolvedValue({ preferred_language: "vi" });
  });

  it("returns current language and options", () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    expect(result.current.currentLanguage).toBe("vi");
    expect(result.current.currentOption.code).toBe("vi");
  });

  it("provides all supported languages", () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    expect(result.current.languages).toHaveLength(3);
    expect(result.current.languages.map((l: any) => l.code)).toEqual(["vi", "en", "zh"]);
    expect(result.current.languages[0]).toHaveProperty("code", "vi");
    expect(result.current.languages[0]).toHaveProperty("nativeName");
    expect(result.current.languages).toEqual(SUPPORTED_LANGUAGES);
  });

  it("changeLanguage calls i18n.changeLanguage", async () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    await act(async () => {
      await result.current.changeLanguage("en");
    });
    expect(mockChangeLanguage).toHaveBeenCalledWith("en");
  });

  it("changeLanguage saves to localStorage", async () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    await act(async () => {
      await result.current.changeLanguage("en");
    });
    expect(localStorageMock.setItem).toHaveBeenCalledWith("i18nextLng", "en");
  });

  it("changeLanguage syncs to server via updateMe", async () => {
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    await act(async () => {
      await result.current.changeLanguage("zh");
    });
    expect(mockUpdateMe).toHaveBeenCalledWith({ preferred_language: "zh" });
  });

  it("changeLanguage handles API errors silently", async () => {
    mockUpdateMe.mockRejectedValueOnce(new Error("API Error"));
    const { result } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    await act(async () => {
      await result.current.changeLanguage("en");
    });
    expect(mockLogError).toHaveBeenCalledWith(
      expect.objectContaining({ message: "API Error" }),
      { component: "useLanguage" },
    );
    expect(mockChangeLanguage).toHaveBeenCalledWith("en");
  });

  it("registers languageChanged event listener on mount", () => {
    renderHook(() => useLanguage(), { wrapper: createWrapper() });
    expect(mockOn).toHaveBeenCalledWith("languageChanged", expect.any(Function));
  });

  it("unregisters languageChanged event listener on unmount", () => {
    const { unmount } = renderHook(() => useLanguage(), { wrapper: createWrapper() });
    unmount();
    expect(mockOff).toHaveBeenCalledWith("languageChanged", expect.any(Function));
  });
});
