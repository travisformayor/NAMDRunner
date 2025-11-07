// Global type definitions for NAMDRunner

interface AppLogger {
  addCommand: (command: string) => void;
  addOutput: (output: string) => void;
  addDebug: (message: string) => void;
}

declare global {
  interface Window {
    appLogger?: AppLogger;
  }
}

export {};