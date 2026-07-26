import { describe, it, expect } from 'vitest';

// Dead exports (formatTime, formatDateTime) were removed from utils.ts.
// Tests for remaining exports (cn, maskEmail, maskPhone, formatBytes) are in
// lib/__tests__/utils.test.ts and utils.pii.test.ts.
describe('Utility Functions', () => {
  it('placeholder — tests for cn, maskEmail, maskPhone, formatBytes live in sibling test files', () => {
    expect(true).toBe(true);
  });
});
