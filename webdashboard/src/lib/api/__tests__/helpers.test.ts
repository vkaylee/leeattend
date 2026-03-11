import { describe, it, expect, beforeEach } from 'vitest';
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

  it('does not throw when localStorage is empty', () => {
    expect(() => clearAuthSession()).not.toThrow();
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
