import { Component, type ReactNode } from 'react';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * React error boundary that catches render errors and displays
 * a user-friendly fallback UI matching the site's glassmorphism design.
 */
export default class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    console.error('[ErrorBoundary] Uncaught error:', error, errorInfo);
  }

  handleReload = (): void => {
    window.location.reload();
  };

  handleGoHome = (): void => {
    window.location.href = '/';
  };

  render(): ReactNode {
    if (!this.state.hasError) {
      return this.props.children;
    }

    return (
      <div className="min-h-screen flex items-center justify-center bg-zinc-50 dark:bg-[#030303] text-zinc-900 dark:text-zinc-50 p-6">
        {/* Ambient background to match Layout */}
        <div className="fixed inset-0 z-0 pointer-events-none overflow-hidden" aria-hidden="true">
          <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] rounded-full bg-indigo-500/20 dark:bg-indigo-500/10 blur-[120px] mix-blend-screen" />
          <div className="absolute top-[20%] right-[-10%] w-[30%] h-[50%] rounded-full bg-violet-500/20 dark:bg-violet-500/10 blur-[120px] mix-blend-screen" />
          <div className="absolute bottom-[-20%] left-[20%] w-[50%] h-[50%] rounded-full bg-blue-500/20 dark:bg-blue-500/10 blur-[120px] mix-blend-screen" />
        </div>

        {/* Glassmorphism error card */}
        <div className="relative z-10 max-w-lg w-full">
          <div className="rounded-2xl border border-white/10 bg-white/5 dark:bg-white/[0.03] backdrop-blur-xl shadow-2xl p-8 sm:p-10 text-center">
            {/* Gradient accent line */}
            <div className="mx-auto mb-6 h-1 w-20 rounded-full bg-gradient-to-r from-indigo-500 via-violet-500 to-blue-500" />

            {/* Icon */}
            <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10 border border-red-500/20">
              <svg
                className="h-8 w-8 text-red-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={1.5}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
                />
              </svg>
            </div>

            <h1 className="text-2xl font-display font-bold mb-2">
              Something went wrong
            </h1>
            <p className="text-zinc-500 dark:text-zinc-400 mb-6 text-sm leading-relaxed">
              An unexpected error occurred while rendering this page.
              You can try reloading the page or navigating back to the home page.
            </p>

            {/* Error details (development aid, collapsed by default) */}
            {this.state.error && (
              <details className="mb-6 text-left">
                <summary className="text-xs text-zinc-400 dark:text-zinc-500 cursor-pointer hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors">
                  Technical details
                </summary>
                <pre className="mt-2 rounded-lg bg-zinc-900/50 border border-white/5 p-3 text-xs text-red-300 overflow-x-auto max-h-32 overflow-y-auto font-mono">
                  {this.state.error.message}
                </pre>
              </details>
            )}

            {/* Action buttons */}
            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <button
                onClick={this.handleReload}
                className="inline-flex items-center justify-center gap-2 rounded-lg bg-gradient-to-r from-indigo-600 to-violet-600 px-5 py-2.5 text-sm font-medium text-white shadow-lg shadow-indigo-500/25 hover:shadow-indigo-500/40 transition-all duration-200 hover:scale-[1.02] active:scale-[0.98]"
              >
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182" />
                </svg>
                Reload page
              </button>
              <button
                onClick={this.handleGoHome}
                className="inline-flex items-center justify-center gap-2 rounded-lg border border-white/10 bg-white/5 px-5 py-2.5 text-sm font-medium text-zinc-300 hover:bg-white/10 transition-all duration-200"
              >
                Go to home
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }
}
