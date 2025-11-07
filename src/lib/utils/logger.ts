/**
 * Centralized logging utility for NAMDRunner
 * All logs display in the Logs Panel in the app
 *
 * Backend logs come through Tauri events (rust-log)
 * Frontend logs use this utility to write to the Logs panel
 */

/**
 * Log a debug/info message
 */
export function debug(tag: string, message: string, error?: any): void {
  const fullMessage = error ? `[${tag}] ${message}: ${error}` : `[${tag}] ${message}`;

  if (typeof window !== 'undefined' && window.appLogger) {
    window.appLogger.addDebug(fullMessage);
  }
}

/**
 * Log an error message
 */
export function error(tag: string, message: string, error?: any): void {
  const fullMessage = error ? `[${tag}] ❌ ${message}: ${error}` : `[${tag}] ❌ ${message}`;

  if (typeof window !== 'undefined' && window.appLogger) {
    window.appLogger.addDebug(fullMessage);
  }
}

/**
 * Log an SSH/SLURM command
 */
export function command(cmd: string): void {
  if (typeof window !== 'undefined' && window.appLogger) {
    window.appLogger.addCommand(cmd);
  }
}

/**
 * Log SSH/SLURM command output
 */
export function output(output: string): void {
  if (typeof window !== 'undefined' && window.appLogger) {
    window.appLogger.addOutput(output);
  }
}

// Default export for convenience
export const logger = {
  debug,
  error,
  command,
  output
};
