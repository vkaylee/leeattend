import { describe, it, expect, beforeEach, afterEach, vi, Mock } from 'vitest';

import {
  getAccessToken,
  setAccessToken,
  getCurrentWorkspaceId,
  setCurrentWorkspaceId,
  getCurrentUserId,
  setCurrentUserId,
  clearAuthSession,
  TIMEOUT,
  apiFetch,
  forceRefreshToken,
  fetchHealth,
} from '../helpers';
import { getFullUrl } from '../api-fetch-helpers';

describe('Token helpers', () => {
  beforeEach(() => {
    // Reset the in-memory store between tests. The store is a module-level
    // singleton, so we wipe via the public API (set to the zero value) rather
    // than touching localStorage — tokens no longer live there.
    setAccessToken(null);
    setCurrentWorkspaceId(null);
    setCurrentUserId(null);
  });

  it('getAccessToken returns null when not set', () => {
    expect(getAccessToken()).toBeNull();
  });

  it('setAccessToken / getAccessToken roundtrip', () => {
    setAccessToken('my-token-123');
    expect(getAccessToken()).toBe('my-token-123');
  });

  it('setAccessToken(null) clears the token', () => {
    setAccessToken('temp');
    setAccessToken(null);
    expect(getAccessToken()).toBeNull();
  });

  // NOTE: there are NO getRefreshToken / setRefreshToken tests anymore.
  // The refresh token is persisted exclusively as an HttpOnly cookie set by
  // the backend; the frontend must never read it.
});

describe('Workspace helpers', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('getCurrentWorkspaceId returns null when not set', () => {
    expect(getCurrentWorkspaceId()).toBeNull();
  });

  it('setCurrentWorkspaceId / getCurrentWorkspaceId roundtrip', () => {
    setCurrentWorkspaceId('ws-789');
    expect(getCurrentWorkspaceId()).toBe('ws-789');
  });
});

describe('User ID helpers', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('getCurrentUserId returns null when not set', () => {
    expect(getCurrentUserId()).toBeNull();
  });

  it('setCurrentUserId / getCurrentUserId roundtrip', () => {
    setCurrentUserId('user-abc');
    expect(getCurrentUserId()).toBe('user-abc');
  });
});

describe('clearAuthSession', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('wipes in-memory credentials', () => {
    setAccessToken('token');
    setCurrentWorkspaceId('ws');
    setCurrentUserId('user');

    clearAuthSession();

    expect(getAccessToken()).toBeNull();
    expect(getCurrentWorkspaceId()).toBeNull();
    expect(getCurrentUserId()).toBeNull();
  });

  it('calls the backend logout endpoint to expire HttpOnly cookies', async () => {
    const mockFetch = globalThis.fetch as Mock;
    clearAuthSession();
    await new Promise(r => setTimeout(r, 0));
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/auth/logout'),
      expect.objectContaining({ method: 'POST', credentials: 'include' }),
    );
  });
});

describe('TIMEOUT constants', () => {
  it('DEFAULT is 10 seconds', () => {
    expect(TIMEOUT.DEFAULT).toBe(10000);
  });

  it('REPORT_OPERATION is 3 minutes', () => {
    expect(TIMEOUT.REPORT_OPERATION).toBe(180000);
  });

  it('QUICK is 5 seconds', () => {
    expect(TIMEOUT.QUICK).toBe(5000);
  });
});

describe('apiFetch interceptor logic', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.stubGlobal('fetch', vi.fn());
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('injects headers correctly', async () => {
    const mockFetch = globalThis.fetch as Mock;
    mockFetch.mockResolvedValue({ ok: true, status: 200 });

    setAccessToken('test-token');
    setCurrentWorkspaceId('ws-123');

    const promise = apiFetch('/v1/test');
    await vi.runAllTimersAsync();
    await promise;

    expect(mockFetch).toHaveBeenCalledTimes(1);
    const [, options] = mockFetch.mock.calls[0];
    
    // Using Headers object
    const headers = options.headers as Headers;
    expect(headers.get('Authorization')).toBe('Bearer test-token');
    expect(headers.get('X-Workspace-Id')).toBe('ws-123');
    expect(headers.get('Content-Type')).toBe('application/json');
  });

  it('injects Accept-Language header correctly based on i18nextLng localStorage', async () => {
    const mockFetch = globalThis.fetch as Mock;
    mockFetch.mockResolvedValue({ ok: true, status: 200 });

    localStorage.setItem('i18nextLng', 'zh-CN');

    const promise = apiFetch('/v1/test');
    await vi.runAllTimersAsync();
    await promise;

    expect(mockFetch).toHaveBeenCalledTimes(1);
    const [, options] = mockFetch.mock.calls[0];
    
    const headers = options.headers as Headers;
    expect(headers.get('Accept-Language')).toBe('zh-CN');
  });

  it('aborts request on timeout', async () => {
    const mockFetch = globalThis.fetch as Mock;
    // Mock fetch that respects the abort signal and rejects when aborted
    mockFetch.mockImplementation((url, options) => new Promise((resolve, reject) => {
      if (options?.signal) {
        if (options.signal.aborted) {
          reject(new DOMException('The user aborted a request.', 'AbortError'));
        } else {
          options.signal.addEventListener('abort', () => {
            reject(new DOMException('The user aborted a request.', 'AbortError'));
          });
        }
      }
    }));

    const promise = apiFetch('/v1/slow', { timeout: 1000 });
    
    // Advance timer by 1001ms
    await vi.advanceTimersByTimeAsync(1001);
    
    // The AbortController should have triggered
    expect(mockFetch).toHaveBeenCalledTimes(1);
    const [, options] = mockFetch.mock.calls[0];
    expect(options.signal.aborted).toBe(true);

    // Await promise to prevent unhandled rejection and use the variable
    await expect(promise).rejects.toThrow();
  });
});

describe('getFullUrl', () => {
  // In the vitest environment import.meta.env.DEV is true, so API_BASE_URL
  // resolves to "/api" (see config.ts). We assert against that contract.

  it('returns absolute http(s) URLs unchanged (covers the early-return branch)', () => {
    expect(getFullUrl('https://example.com/v1/health')).toBe(
      'https://example.com/v1/health',
    );
  });

  it('returns paths already prefixed with API_BASE_URL unchanged', () => {
    expect(getFullUrl('/api/v1/x')).toBe('/api/v1/x');
  });

  it('prepends API_BASE_URL for bare relative paths', () => {
    expect(getFullUrl('v1/health')).toBe('/api/v1/health');
  });

  it('normalises a leading slash before prepending', () => {
    expect(getFullUrl('/v1/health')).toBe('/api/v1/health');
  });
});

describe('forceRefreshToken failure path', () => {
  beforeEach(() => {
    setAccessToken(null);
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('network down')));
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns false and logs when refresh fetch rejects (catch path)', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const result = await forceRefreshToken();
    expect(result).toBe(false);
    expect(errorSpy).toHaveBeenCalledWith(
      'Token refresh failed',
      expect.any(Error),
    );
    errorSpy.mockRestore();
  });
});

describe('handleUnauthorized (401 retry-on-refresh) via apiFetch', () => {
  beforeEach(() => {
    localStorage.clear();
    setAccessToken(null);
    vi.stubGlobal('fetch', vi.fn());
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('retrying once on 401 with a fresh token after forceRefreshToken succeeds', async () => {
    const mockFetch = globalThis.fetch as Mock;
    // 1st call: the original request returns 401
    mockFetch.mockResolvedValueOnce(
      new Response(JSON.stringify({ message: 'unauthorized' }), { status: 401 }),
    );
    // 2nd call: the refresh POST returns a new access_token
    mockFetch.mockResolvedValueOnce(
      new Response(
        JSON.stringify({ access_token: 'new-at', refresh_token: 'new-rt' }),
        { status: 200 },
      ),
    );
    // 3rd call: retry of the original request succeeds
    mockFetch.mockResolvedValueOnce(
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );

    const promise = apiFetch('/v1/secured');
    await vi.runAllTimersAsync();
    const response = await promise;

    expect(mockFetch).toHaveBeenCalledTimes(3);
    // refresh endpoint hit with empty body + credentials:include
    const refreshCall = mockFetch.mock.calls[1];
    expect(refreshCall[0]).toContain('/v1/auth/refresh');
    const refreshOpts = refreshCall[1] as RequestInit;
    expect(refreshOpts.credentials).toBe('include');
    expect(refreshOpts.body).toBe(JSON.stringify({}));
    // retry carried the bearer token
    const retryOpts = mockFetch.mock.calls[2][1] as RequestInit;
    expect((retryOpts.headers as Headers).get('Authorization')).toBe(
      'Bearer new-at',
    );
    expect(response.status).toBe(200);
    expect(getAccessToken()).toBe('new-at');
  });

  it('redirects to login + throws when forceRefreshToken fails on 401', async () => {
    // happy-dom window.location is read-only; gently swap it for an assignable
    // stand-in so we can assert the redirect without triggering navigation.
    const fakeLocation = { pathname: '/workspace', href: '' };
    const originalLocation = window.location;
    Object.defineProperty(window, 'location', {
      value: fakeLocation,
      writable: true,
      configurable: true,
    });

    // Default: every bare fetch resolves to a 200 OK. This also covers the
    // fire-and-forget logout POST inside clearAuthSession (3rd+ calls).
    const mockFetch = globalThis.fetch as Mock;
    mockFetch.mockResolvedValue(
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    // 1st call: original request → 401
    mockFetch.mockResolvedValueOnce(new Response('unauth', { status: 401 }));
    // 2nd call: refresh POST fails (non-OK)
    mockFetch.mockResolvedValueOnce(new Response('boom', { status: 500 }));

    const promise = apiFetch('/v1/secured');
    await vi.runAllTimersAsync();

    await expect(promise).rejects.toThrow('Session expired');
    // redirected somewhere (handleUnauthorized sets window.location.href)
    expect(fakeLocation.href).not.toBe('');
    // credentials wiped
    expect(getAccessToken()).toBeNull();

    Object.defineProperty(window, 'location', {
      value: originalLocation,
      writable: true,
      configurable: true,
    });
  });
});

describe('fetchHealth', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('GETs /v1/health and returns parsed status JSON', async () => {
    const mockFetch = globalThis.fetch as Mock;
    mockFetch.mockResolvedValue(
      new Response(JSON.stringify({ status: 'ok' }), { status: 200 }),
    );
    const result = await fetchHealth();
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(mockFetch.mock.calls[0][0]).toContain('/v1/health');
    expect(result).toEqual({ status: 'ok' });
  });
});

describe('Token Refresh Mutex (Stampede Prevention)', () => {
  beforeEach(() => {
    setAccessToken(null);
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('batches concurrent refresh requests into a single fetch call', async () => {
    const mockFetch = globalThis.fetch as Mock;
    let resolveRefresh: (value: any) => void;
    // Create a delayed promise to simulate network latency for refresh
    const delayedResponse = new Promise((resolve) => {
      resolveRefresh = resolve;
    });

    mockFetch.mockReturnValue(delayedResponse);

    // Call forceRefreshToken 3 times concurrently.
    // The backend reads the refresh_token from an HttpOnly cookie, so the
    // frontend sends an empty body.
    const promises = [
      forceRefreshToken(),
      forceRefreshToken(),
      forceRefreshToken(),
    ];

    // Wait a tick to ensure they all hit the if (refreshPromise) branch
    await new Promise(r => setTimeout(r, 0));

    expect(mockFetch).toHaveBeenCalledTimes(1); // Only ONE network call should be made!
    // Verify the body is EMPTY — the refresh token is NOT sent by the frontend.
    const [, options] = mockFetch.mock.calls[0];
    expect(options.body).toBe(JSON.stringify({}));
    expect(options.credentials).toBe('include'); // so the HttpOnly cookie travels

    // Now resolve the network call
    resolveRefresh!({
      ok: true,
      json: () => Promise.resolve({ access_token: 'new-acc', refresh_token: 'new-ref' }),
    });

    const results = await Promise.all(promises);

    expect(results).toEqual([true, true, true]); // All three should return success
    expect(mockFetch).toHaveBeenCalledTimes(1); // Still exactly 1 fetch call
    expect(getAccessToken()).toBe('new-acc');
  });
});
