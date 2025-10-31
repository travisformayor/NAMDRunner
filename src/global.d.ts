// Global type definitions for NAMDRunner

interface SSHConsole {
  addCommand: (_command: string) => void;
  addOutput: (_output: string) => void;
  addDebug: (_message: string) => void;
}

declare global {
  interface Window {
    sshConsole?: SSHConsole;
  }
}

export {};