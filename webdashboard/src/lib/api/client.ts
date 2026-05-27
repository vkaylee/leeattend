import { apiFetch } from "./helpers";

/**
 * A simple client wrapper around `apiFetch` that handles JSON parsing
 * and throws standard Errors with status codes.
 */
export const apiClient = {
  get: async (url: string) => {
    const res = await apiFetch(url);
    if (!res.ok) {
      const errorText = await res.text().catch(() => "Unknown error");
      throw new Error(`GET ${url} failed: ${res.status} ${res.statusText} - ${errorText}`);
    }
    // Handle empty responses
    const text = await res.text();
    return { data: text ? JSON.parse(text) : null };
  },
  post: async (url: string, body?: any) => {
    const res = await apiFetch(url, {
      method: "POST",
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      const errorText = await res.text().catch(() => "Unknown error");
      throw new Error(`POST ${url} failed: ${res.status} ${res.statusText} - ${errorText}`);
    }
    const text = await res.text();
    return { data: text ? JSON.parse(text) : null };
  },
  put: async (url: string, body?: any) => {
    const res = await apiFetch(url, {
      method: "PUT",
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      const errorText = await res.text().catch(() => "Unknown error");
      throw new Error(`PUT ${url} failed: ${res.status} ${res.statusText} - ${errorText}`);
    }
    const text = await res.text();
    return { data: text ? JSON.parse(text) : null };
  },
  delete: async (url: string) => {
    const res = await apiFetch(url, { method: "DELETE" });
    if (!res.ok) {
      const errorText = await res.text().catch(() => "Unknown error");
      throw new Error(`DELETE ${url} failed: ${res.status} ${res.statusText} - ${errorText}`);
    }
    const text = await res.text();
    return { data: text ? JSON.parse(text) : null };
  },
  patch: async (url: string, body?: any) => {
    const res = await apiFetch(url, {
      method: "PATCH",
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      const errorText = await res.text().catch(() => "Unknown error");
      throw new Error(`PATCH ${url} failed: ${res.status} ${res.statusText} - ${errorText}`);
    }
    const text = await res.text();
    return { data: text ? JSON.parse(text) : null };
  }
};
