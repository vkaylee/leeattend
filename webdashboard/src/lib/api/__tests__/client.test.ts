import { describe, it, expect, beforeEach, afterEach, vi, Mock } from 'vitest';

import { client, customFetch } from '../client';
import { ApiError } from '../errors';

// Mock apiFetch which is used inside client.ts
vi.mock('../helpers', () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from '../helpers';

describe('client (openapi-fetch adapter)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('sanitizes URL correctly to remove redundant /api prefix', async () => {
    const mockApiFetch = apiFetch as Mock;
    // Return a mocked Response
    mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), {
      status: 200,
      statusText: "OK",
    }));

    // In openapi-fetch v7+, fetch adapter might receive a string or Request.
    // The createClient passes the path, so we simulate calling client.GET
    // Since client.GET uses path parameters, we can test by calling it with a fake path.
    // Note: openapi-fetch generates the URL string from baseUrl.
    // Since we mocked apiFetch, we can inspect what URL it receives.
    
    await (client.GET as any)('/api/v1/workspaces', { params: {} });
    
    expect(mockApiFetch).toHaveBeenCalledTimes(1);
    const [url] = mockApiFetch.mock.calls[0];
    
    // Check that there is no double /api
    expect(url).not.toContain('/api/api/v1');
    expect(url).toContain('/api/v1/workspaces');
  });

  it('parses error responses into ApiError correctly', async () => {
    const mockApiFetch = apiFetch as Mock;
    
    const errorBody = {
      error_code: "VALIDATION_FAILED",
      message: "Invalid parameters"
    };

    // Return a mocked 400 Response
    mockApiFetch.mockImplementation(() => Promise.resolve(new Response(JSON.stringify(errorBody), {
      status: 400,
      statusText: "Bad Request",
    })));

    await expect((client.GET as any)('/api/v1/test-error')).rejects.toThrowError(ApiError);

    try {
      await (client.GET as any)('/api/v1/test-error');
    } catch (error) {
      expect(error).toBeInstanceOf(ApiError);
      const apiError = error as ApiError;
      expect(apiError.status).toBe(400);
      expect(apiError.errorCode).toBe('VALIDATION_FAILED');
      expect(apiError.errorBody).toEqual(errorBody);
    }
  });

  it('handles raw text errors gracefully', async () => {
    const mockApiFetch = apiFetch as Mock;
    
    mockApiFetch.mockImplementation(() => Promise.resolve(new Response("Internal Server Error occurred", {
      status: 500,
      statusText: "Internal Server Error",
    })));

    try {
      await (client.GET as any)('/api/v1/test-error-500');
    } catch (error) {
      expect(error).toBeInstanceOf(ApiError);
      const apiError = error as ApiError;
      expect(apiError.status).toBe(500);
      expect(apiError.errorCode).toBeUndefined();
      expect(apiError.errorBody).toEqual({ message: "Internal Server Error occurred" });
    }
  });

  describe('customFetch URL sanitization and cleanup', () => {
    it('handles string URLs without modification', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const testUrl = "http://127.0.0.1:5173/api/v1/workspaces";
      await customFetch(testUrl);

      expect(mockApiFetch).toHaveBeenCalledWith(testUrl, {});
    });

    it('corrects double /api/api/v1 in Request objects', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const request = new Request("http://127.0.0.1:5173/api/api/v1/workspaces", {
        method: "GET",
      });

      await customFetch(request);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "http://127.0.0.1:5173/api/v1/workspaces",
        expect.objectContaining({ method: "GET" })
      );
    });

    it('corrects /api/ missing /v1 in Request objects', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const request = new Request("http://127.0.0.1:5173/api/workspaces", {
        method: "POST",
        body: JSON.stringify({ name: "New" }),
        headers: { "Content-Type": "application/json" },
      });

      await customFetch(request);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "http://127.0.0.1:5173/api/v1/workspaces",
        expect.objectContaining({
          method: "POST",
          headers: expect.objectContaining({ "Content-Type": "application/json" }),
          body: JSON.stringify({ name: "New" }),
        })
      );
    });

    it('leaves correct /api/v1/ paths untouched in Request objects', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const request = new Request("http://127.0.0.1:5173/api/v1/workspaces");
      await customFetch(request);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "http://127.0.0.1:5173/api/v1/workspaces",
        expect.any(Object)
      );
    });

    it('handles Request with FormData (multipart/form-data) body correctly without converting to text', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const formData = new FormData();
      formData.append("file", new Blob(["excel-binary"], { type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" }), "template.xlsx");

      const request = new Request("http://127.0.0.1:5173/api/v1/workspaces/ws-id/leave/upload-excel", {
        method: "POST",
        body: formData,
      });

      await customFetch(request);

      expect(mockApiFetch).toHaveBeenCalledTimes(1);
      const calledOptions = mockApiFetch.mock.calls[0][1];
      expect(calledOptions.method).toBe("POST");
      expect(calledOptions.body).toBeInstanceOf(FormData);
    });

    it('handles Request with binary Blob body correctly without converting to text', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const blob = new Blob(["binary-data"], { type: "application/octet-stream" });
      const request = new Request("http://127.0.0.1:5173/api/v1/workspaces/ws-id/binary-upload", {
        method: "POST",
        body: blob,
        headers: { "Content-Type": "application/octet-stream" },
      });

      await customFetch(request);

      expect(mockApiFetch).toHaveBeenCalledTimes(1);
      const calledOptions = mockApiFetch.mock.calls[0][1];
      expect(calledOptions.method).toBe("POST");
      expect(calledOptions.body).toBeInstanceOf(Blob);
    });

    it('keeps body undefined for POST requests with no body', async () => {
      const mockApiFetch = apiFetch as Mock;
      mockApiFetch.mockResolvedValue(new Response(JSON.stringify({ data: "ok" }), { status: 200 }));

      const request = new Request("http://127.0.0.1:5173/api/v1/workspaces/ws-id/trigger-job", {
        method: "POST",
      });

      await customFetch(request);

      expect(mockApiFetch).toHaveBeenCalledTimes(1);
      const calledOptions = mockApiFetch.mock.calls[0][1];
      expect(calledOptions.method).toBe("POST");
      expect(calledOptions.body).toBeUndefined();
    });
  });
});
