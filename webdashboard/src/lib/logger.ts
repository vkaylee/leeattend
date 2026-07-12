import * as Sentry from "@sentry/react";

interface ErrorContext {
  /** Component or module where the error occurred */
  component?: string;
  /** Additional key-value metadata attached to the Sentry event */
  extra?: Record<string, unknown>;
}

/**
 * Centralized error logger that reports to Sentry when initialized,
 * and falls back to `console.error` in development when Sentry is unavailable.
 *
 * @example
 * try {
 *   await mutateSomething();
 * } catch (e) {
 *   logError(e, { component: "EmployeeImport", extra: { rowCount } });
 *   toast.error("Import failed");
 * }
 */
export function logError(error: unknown, context?: ErrorContext): void {
  if (context?.component || context?.extra) {
    Sentry.setContext("app", {
      component: context.component,
      ...context.extra,
    });
  }

  if (Sentry.isInitialized()) {
    Sentry.captureException(error);
  } else if (import.meta.env.DEV) {
    // eslint-disable-next-line no-console
    console.error(`[${context?.component ?? "logError"}]`, error);
  }
}
