import React from "react";

import { ErrorBoundary } from "./error-boundary";

interface ResilientWidgetProps {
  children: React.ReactNode;
  /** Custom fallback component if the default ErrorBoundary layout isn't desired */
  fallback?: React.ReactNode;
  /** Callback triggered when a rendering error is caught */
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  /** Error boundary display layout variant */
  variant?: "full" | "compact";
}

/**
 * ResilientWidget acts as a safety boundary for independent UI components.
 * If a rendering crash occurs within its scope, it isolates the failure and
 * displays a localized error view rather than crashing the parent page.
 */
export function ResilientWidget({
  children,
  fallback,
  onError,
  variant = "compact",
}: ResilientWidgetProps) {
  return (
    <ErrorBoundary fallback={fallback} onError={onError} variant={variant}>
      {children}
    </ErrorBoundary>
  );
}
