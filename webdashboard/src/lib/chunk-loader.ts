const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1000;

export function retryImport<T>(
  importer: () => Promise<T>,
  retries = MAX_RETRIES,
  delay = RETRY_DELAY_MS,
): Promise<T> {
  return importer().catch((error: unknown) => {
    if (retries <= 0) throw error;
    return new Promise<T>((resolve) =>
      setTimeout(
        () => { resolve(retryImport(importer, retries - 1, delay * 2)); },
        delay,
      ),
    );
  });
}

export function lazyRetry<T>(
  importer: () => Promise<{ default: T }>,
  retries = MAX_RETRIES,
): () => Promise<{ default: T }> {
  return () => retryImport(importer, retries);
}
