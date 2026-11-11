import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from "vitest";

import common from "@/i18n/locales/vi/common.json";

import { useFileUpload, type UseFileUploadOptions } from "../use-file-upload";

// The setup's react-i18next mock resolves keys against the merged vi locale,
// so errors.* keys translate to real Vietnamese strings.
const ERROR_MSG = common.errors;

const clientMock = vi.hoisted(() => ({ POST: vi.fn() }));
const legacyFetchMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/client", () => ({
  client: clientMock,
  fetch: legacyFetchMock,
}));

const hashMock = vi.hoisted(() => ({ computeSHA256: vi.fn() }));
vi.mock("@/utils/hash", () => hashMock);

const defaultOptions: UseFileUploadOptions = {
  category: "onboarding",
  ownerType: "employee",
  ownerId: "emp-1",
};

class FakeXHR {
  static instances: FakeXHR[] = [];
  static status = 200;
  static behavior: "load" | "error" | "manual" = "load";

  upload: { onprogress: ((e: ProgressEvent) => void) | null } = { onprogress: null };
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  status = 200;
  open = vi.fn();
  setRequestHeader = vi.fn();
  send = vi.fn((_body: unknown) => {
    this.status = FakeXHR.status;
    if (FakeXHR.behavior === "load") this.onload?.();
    else if (FakeXHR.behavior === "error") this.onerror?.();
  });

  constructor() {
    FakeXHR.instances.push(this);
  }
}

function makeFile(size: number, name = "file.bin", type = "application/octet-stream"): File {
  const file = new File([new Uint8Array(1)], name, { type });
  Object.defineProperty(file, "size", { value: size });
  return file;
}

const SMALL = 1024;
const LARGE = 6 * 1024 * 1024;

let queryClient: QueryClient;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

describe("useFileUpload", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    hashMock.computeSHA256.mockResolvedValue("hash123");
    FakeXHR.instances = [];
    FakeXHR.status = 200;
    FakeXHR.behavior = "load";
    vi.stubGlobal("XMLHttpRequest", FakeXHR);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe("direct upload (small files)", () => {
    it("uploads via POST with a FormData payload and reports 100% progress", async () => {
      const dbFile = { id: "file-1", file_name: "doc.pdf" };
      (clientMock.POST as Mock).mockResolvedValue({ data: dbFile, error: undefined });
      const onSuccess = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onSuccess }),
        { wrapper: createWrapper() },
      );

      await act(async () => {
        await result.current.uploadAsync(makeFile(SMALL, "doc.pdf", "application/pdf"));
      });

      expect(hashMock.computeSHA256).toHaveBeenCalledTimes(1);
      expect(clientMock.POST).toHaveBeenCalledTimes(1);

      const [url, init] = (clientMock.POST as Mock).mock.calls[0];
      expect(url).toBe("/api/v1/files/upload");

      const formData = init.body as FormData;
      expect(formData.get("category")).toBe("onboarding");
      expect(formData.get("owner_type")).toBe("employee");
      expect(formData.get("owner_id")).toBe("emp-1");
      expect(formData.get("content_hash")).toBe("hash123");
      expect(formData.get("file")).toBeInstanceOf(File);

      expect(onSuccess).toHaveBeenCalledWith(dbFile);
      expect(result.current.progress).toBe(100);
      expect(result.current.error).toBeNull();
      expect(legacyFetchMock).not.toHaveBeenCalled();
    });

    it("throws the backend error message and invokes onError", async () => {
      (clientMock.POST as Mock).mockRejectedValue(new Error("storage full"));
      const onError = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onError }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      act(() => {
        promise = result.current.uploadAsync(makeFile(SMALL));
      });
      await act(async () => {
        await expect(promise).rejects.toThrow("storage full");
      });

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
      expect(result.current.error?.message).toBe("storage full");
    });

    it("falls back to the i18n message when the error has no message", async () => {
      (clientMock.POST as Mock).mockRejectedValue(new Error());
      const onError = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onError }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      act(() => {
        promise = result.current.uploadAsync(makeFile(SMALL));
      });
      await act(async () => {
        await expect(promise).rejects.toThrow(ERROR_MSG.upload_failed);
      });

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
    });

    it("toggles isUploading while the upload is pending", async () => {
      let resolveUpload: ((value: { data: unknown; error: undefined }) => void) | undefined;
      const deferred = new Promise((resolve) => { resolveUpload = resolve; });
      (clientMock.POST as Mock).mockReturnValue(deferred);

      const { result } = renderHook(() => useFileUpload(defaultOptions), { wrapper: createWrapper() });

      act(() => {
        result.current.upload(makeFile(SMALL));
      });
      await waitFor(() => expect(result.current.isUploading).toBe(true));

      await act(async () => {
        resolveUpload?.({ data: { id: "file-1" }, error: undefined });
      });
      await waitFor(() => expect(result.current.isUploading).toBe(false));
    });
  });

  describe("presigned upload (large files)", () => {
    const dbFile = { id: "file-2", file_name: "big.zip" };

    it("requests a presigned URL, PUTs the file and returns the stored file", async () => {
      (legacyFetchMock as Mock).mockResolvedValue({
        data: { file: dbFile, upload_url: "https://s3.example/upload" },
      });
      const onSuccess = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onSuccess }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      await act(async () => {
        promise = result.current.uploadAsync(makeFile(LARGE, "big.zip"));
      });
      await waitFor(() => expect(FakeXHR.instances.length).toBeGreaterThan(0));

      const [method, url, init] = (legacyFetchMock as Mock).mock.calls[0];
      expect(method).toBe("POST");
      expect(url).toBe("/api/v1/files/presigned");
      expect(init.body).toMatchObject({
        category: "onboarding",
        content_hash: "hash123",
        file_name: "big.zip",
        file_size: LARGE,
        mime_type: "application/octet-stream",
        owner_id: "emp-1",
        owner_type: "employee",
      });

      const xhr = FakeXHR.instances[0];
      expect(xhr.open).toHaveBeenCalledWith("PUT", "https://s3.example/upload", true);
      expect(xhr.setRequestHeader).toHaveBeenCalledWith("Content-Type", "application/octet-stream");
      expect(xhr.send).toHaveBeenCalledWith(expect.any(File));

      await act(async () => {
        await promise;
      });

      expect(onSuccess).toHaveBeenCalledWith(dbFile);
      expect(clientMock.POST).not.toHaveBeenCalled();
    });

    it("tracks upload progress from the XHR progress events", async () => {
      FakeXHR.behavior = "manual";
      (legacyFetchMock as Mock).mockResolvedValue({
        data: { file: dbFile, upload_url: "https://s3.example/upload" },
      });
      const onSuccess = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onSuccess }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      await act(async () => {
        promise = result.current.uploadAsync(makeFile(LARGE));
      });
      await waitFor(() => expect(FakeXHR.instances.length).toBeGreaterThan(0));

      const xhr = FakeXHR.instances[0];
      act(() => {
        xhr.upload.onprogress?.({ lengthComputable: true, loaded: 50, total: 100 } as ProgressEvent);
      });
      expect(result.current.progress).toBe(50);

      act(() => {
        xhr.upload.onprogress?.({ lengthComputable: true, loaded: 100, total: 100 } as ProgressEvent);
      });
      expect(result.current.progress).toBe(100);

      await act(async () => {
        xhr.onload?.();
      });
      await act(async () => {
        await promise;
      });
      expect(onSuccess).toHaveBeenCalledWith(dbFile);
    });

    it("rejects when the presigned request fails and invokes onError", async () => {
      (legacyFetchMock as Mock).mockRejectedValue(new Error("presigned-boom"));
      const onError = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onError }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      act(() => {
        promise = result.current.uploadAsync(makeFile(LARGE));
      });
      await act(async () => {
        await expect(promise).rejects.toThrow("presigned-boom");
      });

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
    });

    it("rejects when the presigned response body is null", async () => {
      (legacyFetchMock as Mock).mockResolvedValue({ data: null });
      const onError = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onError }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      act(() => {
        promise = result.current.uploadAsync(makeFile(LARGE));
      });
      await act(async () => {
        await expect(promise).rejects.toThrow(ERROR_MSG.presigned_url_failed);
      });

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
    });

    it("rejects with the s3 error when the PUT returns a non-2xx status", async () => {
      FakeXHR.behavior = "manual";
      FakeXHR.status = 500;
      (legacyFetchMock as Mock).mockResolvedValue({
        data: { file: dbFile, upload_url: "https://s3.example/upload" },
      });
      const onError = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onError }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      await act(async () => {
        promise = result.current.uploadAsync(makeFile(LARGE));
      });
      await waitFor(() => expect(FakeXHR.instances.length).toBeGreaterThan(0));

      const xhr = FakeXHR.instances[0];
      xhr.status = 500;
      await act(async () => {
        xhr.onload?.();
      });
      await act(async () => {
        await expect(promise).rejects.toThrow(ERROR_MSG.s3_upload_failed);
      });

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
    });

    it("rejects with the network error when the PUT fails to send", async () => {
      FakeXHR.behavior = "manual";
      (legacyFetchMock as Mock).mockResolvedValue({
        data: { file: dbFile, upload_url: "https://s3.example/upload" },
      });
      const onError = vi.fn();
      const { result } = renderHook(
        () => useFileUpload({ ...defaultOptions, onError }),
        { wrapper: createWrapper() },
      );

      let promise!: Promise<unknown>;
      await act(async () => {
        promise = result.current.uploadAsync(makeFile(LARGE));
      });
      await waitFor(() => expect(FakeXHR.instances.length).toBeGreaterThan(0));

      const xhr = FakeXHR.instances[0];
      await act(async () => {
        xhr.onerror?.();
      });
      await act(async () => {
        await expect(promise).rejects.toThrow(ERROR_MSG.network_error);
      });

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
    });
  });
});
