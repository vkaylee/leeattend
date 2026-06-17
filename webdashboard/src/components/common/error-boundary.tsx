import React, { Component } from "react";

import i18n from "i18next";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface ErrorBoundaryProps {
  children?: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  variant?: "full" | "compact";
}

interface ErrorBoundaryState {
  error: Error | null;
  hasError: boolean;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error("[ErrorBoundary]", error, errorInfo);
    this.props.onError?.(error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  handleHardReload = () => {
    window.location.reload();
  };

  renderCompactFallback() {
    return (
      <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm flex flex-col items-center justify-center min-h-[300px] text-center space-y-4">
        <div className="w-12 h-12 rounded-full bg-red-50 dark:bg-red-950/30 flex items-center justify-center text-red-500">
          <svg className="w-6 h-6 text-red-500" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" x2="12" y1="8" y2="12" />
            <line x1="12" x2="12.01" y1="16" y2="16" />
          </svg>
        </div>
        <div>
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wider" variant="h4">
            {i18n.t("common.error_boundary.widget_error")}
          </Typography>
          <Typography className="text-[11px] text-slate-400 dark:text-slate-500 mt-1 max-w-xs" variant="p">
            {this.state.error?.message ?? i18n.t("common.error_boundary.unexpected_error")}
          </Typography>
        </div>
        <Button className="flex items-center gap-2 px-4 py-2 bg-slate-50 dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-xl border border-slate-100 dark:border-slate-800 transition-all text-xs font-semibold active:scale-95"
          onClick={this.handleRetry}
          variant="ghost"
        >
          {i18n.t("common.retry")}
        </Button>
      </div>
    );
  }

  renderFullFallback() {
    return (
      <div className="min-h-[400px] flex items-center justify-center p-8">
        <div className="max-w-md text-center space-y-4">
          <div className="w-16 h-16 mx-auto rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
            <svg className="w-8 h-8 text-red-500" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" x2="9" y1="9" y2="15" />
              <line x1="9" x2="15" y1="9" y2="15" />
            </svg>
          </div>
          <Typography className="text-lg font-semibold text-slate-800 dark:text-slate-200" variant="h2">
            {i18n.t("common.error_boundary.something_went_wrong")}
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
            {this.state.error?.message ?? i18n.t("common.error_boundary.unexpected_error")}
          </Typography>
          <div className="flex justify-center gap-3">
            <Button className="px-4 py-2 text-sm font-medium rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors"
              onClick={this.handleRetry}
              variant="ghost"
            >
              {i18n.t("common.retry")}
            </Button>
            <Button className="px-4 py-2 text-sm font-medium rounded-lg border border-slate-300 dark:border-slate-600 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
              onClick={this.handleHardReload}
              variant="ghost"
            >
              {i18n.t("common.error_boundary.reload_page")}
            </Button>
          </div>
          {this.state.error && import.meta.env.DEV ? (
            <pre className="mt-4 p-4 text-left text-xs bg-slate-100 dark:bg-slate-900 rounded-lg overflow-auto max-h-[200px] text-red-600 dark:text-red-400">
              {this.state.error.stack}
            </pre>
          ) : null}
        </div>
      </div>
    );
  }

  render() {
    const { hasError } = this.state;
    const { fallback, variant, children } = this.props;

    if (!hasError) {
      return children ?? null;
    }

    if (fallback) {
      return fallback;
    }

    return variant === "compact" ? this.renderCompactFallback() : this.renderFullFallback();
  }
}

