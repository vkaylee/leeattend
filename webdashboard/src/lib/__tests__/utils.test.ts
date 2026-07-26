import { describe, it, expect } from 'vitest';

// Dead exports (formatTime, formatDateTime, isAgentOnline) were removed from utils.ts.
// Tests for remaining exports (cn, maskEmail, maskPhone, formatBytes) are in
// utils.test.ts and utils.pii.test.ts at the lib/ level.
describe('utils', () => {
  it('placeholder — active exports are tested in sibling test files', () => {
    expect(true).toBe(true);
  });
});
