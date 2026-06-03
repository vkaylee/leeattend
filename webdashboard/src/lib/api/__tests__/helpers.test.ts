import { describe, it, expect, beforeEach, afterEach, vi, Mock } from 'vitest';

import {
  getAccessToken,
  setAccessToken,
  getRefreshToken,
  setRefreshToken,
  getCurrentWorkspaceId,
  setCurrentWorkspaceId,
  getCurrentUserId,
  setCurrentUserId,
  clearAuthSession,
  TIMEOUT,
  apiFetch,
  forceRefreshToken,
} from '../helpers';

describe('Token helpers', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('getAccessToken returns null when not set', () => {
    expect(getAccessToken()).toBeNull();
  });

  it('setAccessToken / getAccessToken roundtrip', () => {
    setAccessToken('my-token-123');
    expect(getAccessToken()).toBe('my-token-123');
  });

  it('getRefreshToken returns null when not set', () => {
    expect(getRefreshToken()).toBeNull();
  });

  it('setRefreshToken / getRefreshToken roundtrip', () => {
    setRefreshToken('refresh-456');
    expect(getRefreshToken()).toBe('refresh-456');
  });
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
    localStorage.clear();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('clears all auth-related localStorage keys', () => {
    setAccessToken('token');
    setRefreshToken('refresh');
    setCurrentWorkspaceId('ws');
    setCurrentUserId('user');

    clearAuthSession();

    expect(getAccessToken()).toBeNull();
    expect(getRefreshToken()).toBeNull();
    expect(getCurrentWorkspaceId()).toBeNull();
    expect(getCurrentUserId()).toBeNull();
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

describe('Token Refresh Mutex (Stampede Prevention)', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('batches concurrent refresh requests into a single fetch call', async () => {
    setRefreshToken('refresh-abc');

    const mockFetch = globalThis.fetch as Mock;
    let resolveRefresh: (value: any) => void;
    // Create a delayed promise to simulate network latency for refresh
    const delayedResponse = new Promise((resolve) => {
      resolveRefresh = resolve;
    });

    mockFetch.mockReturnValue(delayedResponse);

    // Call forceRefreshToken 3 times concurrently
    const promises = [
      forceRefreshToken(),
      forceRefreshToken(),
      forceRefreshToken(),
    ];

    // Wait a tick to ensure they all hit the if (refreshPromise) branch
    await new Promise(r => setTimeout(r, 0));
    
    expect(mockFetch).toHaveBeenCalledTimes(1); // Only ONE network call should be made!

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
