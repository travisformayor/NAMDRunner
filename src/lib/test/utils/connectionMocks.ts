import type {
  SSHConnection,
  SFTPConnection,
  CommandResult,
  FileListResult,
  DirectoryResult
} from '../../types/connection';

/**
 * Simple SSH Mock for Testing
 * Replaces complex 190+ line mock implementations with focused test doubles
 */
export class MockSSHConnection implements SSHConnection {
  private connected: boolean;
  private responses: Map<string, CommandResult>;

  constructor(connected = true, customResponses: Record<string, CommandResult> = {}) {
    this.connected = connected;
    this.responses = new Map(Object.entries(customResponses));
    this.setupDefaultResponses();
  }

  private setupDefaultResponses() {
    const defaults: Record<string, CommandResult> = {
      'echo "test"': { stdout: 'test', stderr: '', exitCode: 0, duration: 50, timedOut: false },
      'echo "health_check"': { stdout: 'health_check', stderr: '', exitCode: 0, duration: 50, timedOut: false },
      'echo $0': { stdout: 'bash', stderr: '', exitCode: 0, duration: 50, timedOut: false },
      'pwd': { stdout: '/home/testuser', stderr: '', exitCode: 0, duration: 50, timedOut: false },
      'module --version': { stdout: 'Modules 4.8.0', stderr: '', exitCode: 0, duration: 200, timedOut: false },
      'sinfo --version': { stdout: 'slurm 21.08.8', stderr: '', exitCode: 0, duration: 200, timedOut: false },
      'sinfo -h -o "%P"': { stdout: 'normal\nhigh\nGPU\n', stderr: '', exitCode: 0, duration: 200, timedOut: false }
    };

    for (const [cmd, result] of Object.entries(defaults)) {
      if (!this.responses.has(cmd)) {
        this.responses.set(cmd, result);
      }
    }
  }

  async connect(): Promise<any> {
    this.connected = true;
    return { success: true, data: {} };
  }

  async disconnect(): Promise<any> {
    this.connected = false;
    return { success: true, data: undefined };
  }

  async executeCommand(command: string): Promise<{ success: boolean; data: CommandResult }> {
    if (!this.connected) {
      return { success: false, data: {} as CommandResult };
    }

    // Check for exact match first
    if (this.responses.has(command)) {
      return { success: true, data: this.responses.get(command)! };
    }

    // Pattern matching for common command types
    if (command.includes('touch') || command.includes('mkdir') || command.includes('rm -rf')) {
      return {
        success: true,
        data: { stdout: '', stderr: '', exitCode: 0, duration: 100, timedOut: false }
      };
    }

    if (command.includes('help')) {
      return {
        success: true,
        data: { stdout: 'Usage: ...', stderr: '', exitCode: 0, duration: 100, timedOut: false }
      };
    }

    // Default success response
    return {
      success: true,
      data: { stdout: 'mock output', stderr: '', exitCode: 0, duration: 100, timedOut: false }
    };
  }

  async validateConnection(): Promise<any> {
    return { success: this.connected, data: this.connected };
  }

  getStatus(): any {
    return this.connected ? 'Connected' : 'Disconnected';
  }

  isConnected(): boolean {
    return this.connected;
  }

  // Test helper methods
  setConnected(connected: boolean) {
    this.connected = connected;
  }

  addCommandResponse(command: string, result: CommandResult) {
    this.responses.set(command, result);
  }
}

/**
 * Simple SFTP Mock for Testing
 */
export class MockSFTPConnection implements SFTPConnection {
  private shouldSucceed: boolean;

  constructor(shouldSucceed = true) {
    this.shouldSucceed = shouldSucceed;
  }

  async uploadFile(): Promise<any> {
    return { success: this.shouldSucceed, data: {} };
  }

  async downloadFile(): Promise<any> {
    return { success: this.shouldSucceed, data: {} };
  }

  async listFiles(remotePath: string): Promise<{ success: boolean; data: FileListResult }> {
    if (!this.shouldSucceed) {
      return { success: false, data: {} as FileListResult };
    }

    return {
      success: true,
      data: {
        files: [{
          name: 'test.txt',
          path: `${remotePath}/test.txt`,
          size: 1024,
          modifiedAt: new Date().toISOString(),
          permissions: 'rw-r--r--',
          isDirectory: false
        }],
        totalCount: 1,
        path: remotePath
      }
    };
  }

  async createDirectory(): Promise<{ success: boolean; data: DirectoryResult }> {
    return {
      success: this.shouldSucceed,
      data: { path: '/test/path', created: true, existed: false }
    };
  }

  async deleteFile(): Promise<any> {
    return { success: this.shouldSucceed, data: {} };
  }

  async exists(): Promise<any> {
    return { success: true, data: true };
  }

  async getFileInfo(): Promise<any> {
    return { success: this.shouldSucceed, data: {} };
  }

  // Test helper methods
  setShouldSucceed(shouldSucceed: boolean) {
    this.shouldSucceed = shouldSucceed;
  }
}

/**
 * Mock Factory for common test scenarios
 */
export const ConnectionMocks = {
  // Working SSH connection with standard responses
  workingSSH: () => new MockSSHConnection(true),

  // Disconnected SSH connection
  disconnectedSSH: () => new MockSSHConnection(false),

  // SSH connection with custom command responses
  customSSH: (responses: Record<string, CommandResult>) => new MockSSHConnection(true, responses),

  // Working SFTP connection
  workingSFTP: () => new MockSFTPConnection(true),

  // Failing SFTP connection
  failingSFTP: () => new MockSFTPConnection(false)
};