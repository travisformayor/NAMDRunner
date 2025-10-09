// Global type definitions for NAMDRunner

interface SSHConsole {
  addCommand: (command: string) => void;
  addOutput: (output: string) => void;
  addDebug: (message: string) => void;
}

declare global {
  interface Window {
    sshConsole?: SSHConsole;
  }
}

export {};