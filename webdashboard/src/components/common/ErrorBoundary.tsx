import React, { Component } from "react";

interface ErrorBoundaryProps {
  children?: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
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

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

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
            <h2 className="text-lg font-semibold text-slate-800 dark:text-slate-200">
              Something went wrong
            </h2>
            <p className="text-sm text-slate-500 dark:text-slate-400">
              {this.state.error?.message || "An unexpected error occurred"}
            </p>
            <div className="flex justify-center gap-3">
              <button
                className="px-4 py-2 text-sm font-medium rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                onClick={this.handleRetry}
              >
                Try again
              </button>
              <button
                className="px-4 py-2 text-sm font-medium rounded-lg border border-slate-300 dark:border-slate-600 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                onClick={this.handleHardReload}
              >
                Reload page
              </button>
            </div>
            {!!this.state.error && !!import.meta.env.DEV && (
              <pre className="mt-4 p-4 text-left text-xs bg-slate-100 dark:bg-slate-900 rounded-lg overflow-auto max-h-[200px] text-red-600 dark:text-red-400">
                {this.state.error.stack}
              </pre>
            )}
          </div>
        </div>
      );
    }

    return this.props.children ?? null;
  }
}
