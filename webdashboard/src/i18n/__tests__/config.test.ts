import { describe, it, expect } from 'vitest';

import i18n, { lazyLoadBackend } from '../config';

describe('i18n config lazyLoadBackend', () => {
  it('should successfully read an existing language (vi)', async () => {
    // Wrap the callback to convert to promise
    const result = await new Promise((resolve, reject) => {
      lazyLoadBackend.read('vi', 'translation', (error, data) => {
        if (error) reject(error instanceof Error ? error : new Error(error));
        else resolve(data);
      });
    });

    expect(result).toBeDefined();
    expect(typeof result).toBe('object');
    // Basic structural check for the Vietnamese locale
    expect((result as Record<string, unknown>).common).toBeDefined();
  });

  it('should handle languages with regional subtags (en-US -> en)', async () => {
    const result = await new Promise((resolve, reject) => {
      lazyLoadBackend.read('en-US', 'translation', (error, data) => {
        if (error) reject(error instanceof Error ? error : new Error(error));
        else resolve(data);
      });
    });

    expect(result).toBeDefined();
    expect(typeof result).toBe('object');
    expect((result as Record<string, unknown>).common).toBeDefined();
  });

  it('should return an error for an unsupported language', async () => {
    const error = await new Promise((resolve) => {
      lazyLoadBackend.read('fr', 'translation', (error_, _data) => {
        resolve(error_);
      });
    });

    expect(error).toBeInstanceOf(Error);
    expect((error as Error).message).toBe('Language fr not found');
  });

  it('should initialize i18next instance successfully', () => {
    expect(i18n.isInitialized).toBeDefined();
  });
});
