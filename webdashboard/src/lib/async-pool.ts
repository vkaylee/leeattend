// Runs an async worker over items with a bounded number of concurrent
// in-flight tasks, settling every item instead of aborting on first failure.
// Used to cap the request fan-out of bulk roster operations, which otherwise
// fire one request per employee all at once and overwhelm the backend.

export interface PoolResult<T> {
  error: unknown;
  item: T;
}

export interface PoolCallbacks<T> {
  /**
   * Called after `worker(item)` settles, in both success and failure paths.
   * `error` is `undefined` on success, otherwise the thrown value.
   */
  onItemSettle?: (item: T, index: number, error: unknown) => void;
  /** Called just before `worker(item)` starts. */
  onItemStart?: (item: T, index: number) => void;
}

/**
 * Run `worker` over `items` with at most `concurrency` tasks in flight.
 *
 * Unlike `Promise.all`, one rejected task does not cancel the rest: every item
 * is awaited and its outcome recorded. Returns the items that failed, each with
 * the error thrown, in completion order.
 *
 * `callbacks` lets callers observe per-item lifecycle (e.g. to drive per-row UI
 * status) without waiting for the whole pool to settle.
 */
export async function runPool<T>(
  items: readonly T[],
  concurrency: number,
  worker: (item: T) => Promise<void>,
  callbacks?: PoolCallbacks<T>,
): Promise<PoolResult<T>[]> {
  const limit = Math.max(1, Math.floor(concurrency));
  const failures: PoolResult<T>[] = [];
  let next = 0;

  async function runWorker(): Promise<void> {
    while (next < items.length) {
      const index = next;
      next += 1;
      const item = items[index];
      callbacks?.onItemStart?.(item, index);
      try {
        await worker(item);
        callbacks?.onItemSettle?.(item, index, undefined);
      } catch (error) {
        failures.push({ item, error });
        callbacks?.onItemSettle?.(item, index, error);
      }
    }
  }

  const workerCount = Math.min(limit, items.length);
  await Promise.all(Array.from({ length: workerCount }, () => runWorker()));
  return failures;
}
