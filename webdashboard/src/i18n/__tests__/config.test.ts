import { describe, it, expect } from 'vitest';

import i18n, { lazyLoadBackend } from '../config';

describe('i18n config lazyLoadBackend', () => {
  it('should successfully read an existing language (vi)', async () => {
    // Wrap the callback to convert to promise
    const result = await new Promise((resolve, reject) => {
      lazyLoadBackend.read('vi', 'common', (error, data) => {
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
      lazyLoadBackend.read('en-US', 'common', (error, data) => {
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
      lazyLoadBackend.read('fr', 'common', (error_, _data) => {
        resolve(error_);
      });
    });

    expect(error).toBeInstanceOf(Error);
    expect((error as Error).message).toBe('Language fr not found');
  });

  it('should initialize i18next instance successfully', async () => {
    if (!i18n.isInitialized) {
      await new Promise<void>((resolve) => {
        i18n.on('initialized', () => {
          resolve();
        });
      });
    }
    expect(i18n.isInitialized).toBe(true);
  });

  it('should resolve keys dynamically using custom t override', async () => {
    await i18n.loadNamespaces(['common', 'payroll']);
    

    // Verify payroll namespace key resolves through override
    const calculateBtn = i18n.t('payroll.calculate_btn');
    expect(calculateBtn).not.toBe('payroll.calculate_btn');
    expect(typeof calculateBtn).toBe('string');
  });
});
