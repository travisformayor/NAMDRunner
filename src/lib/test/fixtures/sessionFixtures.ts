import type { 
  ConnectParams, 
  ConnectResult, 
  DisconnectResult, 
  ConnectionStatusResult,
  SessionInfo,
  ConnectionState 
} from '../../types/api';

/**
 * Session and connection fixture utilities for testing
 * Covers various connection scenarios including success, failure, and edge cases
 */

export interface ConnectionScenario {
  name: string;
  description: string;
  connectParams: ConnectParams;
  expectedResult: ConnectResult;
  expectedDelay?: number; // Simulated network delay in ms
}

export interface SessionStateScenario {
  name: string;
  description: string;
  state: ConnectionState;
  sessionInfo?: SessionInfo;
}

// Mock connection parameters for testing  
export const clusterHosts = {
  mock: 'mock.cluster.test',
  invalid: 'invalid.cluster.test',
};

// Mock usernames for testing
export const testUsers = {
  valid: 'mockuser',
  invalid: 'invaliduser',
};

// Connection scenario fixtures - simplified for practical testing
export const connectionFixtures: Record<string, ConnectionScenario> = {
  successfulConnection: {
    name: 'Successful Connection',
    description: 'Normal successful connection to mock cluster',
    connectParams: {
      host: clusterHosts.mock,
      username: testUsers.valid,
      password: 'correct_password',
    },
    expectedResult: {
      success: true,
      sessionInfo: {
        host: clusterHosts.mock,
        username: testUsers.valid,
        connectedAt: new Date().toISOString(),
      },
    },
    expectedDelay: 500,
  },

  wrongPassword: {
    name: 'Wrong Password',
    description: 'Authentication fails due to incorrect password',
    connectParams: {
      host: clusterHosts.mock,
      username: testUsers.valid,
      password: 'wrong_password',
    },
    expectedResult: {
      success: false,
      error: 'Authentication failed: Invalid password',
    },
    expectedDelay: 300,
  },

  hostUnreachable: {
    name: 'Host Unreachable',
    description: 'Connection fails because host is unreachable',
    connectParams: {
      host: clusterHosts.invalid,
      username: testUsers.valid,
      password: 'any_password',
    },
    expectedResult: {
      success: false,
      error: 'Connection failed: Host unreachable',
    },
    expectedDelay: 1000,
  },
};

// Session state fixtures for testing UI state management  
export const sessionStateFixtures: Record<string, SessionStateScenario> = {
  disconnected: {
    name: 'Disconnected State',
    description: 'User is not connected to any cluster',
    state: 'Disconnected',
  },

  connected: {
    name: 'Connected State',
    description: 'Successfully connected to mock cluster',
    state: 'Connected',
    sessionInfo: {
      host: clusterHosts.mock,
      username: testUsers.valid,
      connectedAt: new Date(Date.now() - 10 * 60 * 1000).toISOString(),
    },
  },

  expired: {
    name: 'Expired Session',
    description: 'Session expired due to inactivity',
    state: 'Expired',
    sessionInfo: {
      host: clusterHosts.mock,
      username: testUsers.valid,
      connectedAt: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    },
  },
};

// Disconnect scenarios
export const disconnectFixtures: Record<string, { name: string; expectedResult: DisconnectResult }> = {
  gracefulDisconnect: {
    name: 'Graceful Disconnect',
    expectedResult: {
      success: true,
    },
  },

  alreadyDisconnected: {
    name: 'Already Disconnected',
    expectedResult: {
      success: true, // Should succeed even if already disconnected
    },
  },

  networkErrorDuringDisconnect: {
    name: 'Network Error During Disconnect',
    expectedResult: {
      success: false,
      error: 'Failed to properly close connection: Network error',
    },
  },
};

// Connection status check scenarios - simplified
export const statusCheckFixtures: Record<string, { name: string; expectedResult: ConnectionStatusResult }> = {
  activeSession: {
    name: 'Active Session Status',
    expectedResult: {
      state: 'Connected',
      sessionInfo: {
        host: clusterHosts.mock,
        username: testUsers.valid,
        connectedAt: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
      },
    },
  },

  noSession: {
    name: 'No Active Session',
    expectedResult: {
      state: 'Disconnected',
    },
  },
};

// Helper functions for test scenarios
export function getConnectionScenario(name: keyof typeof connectionFixtures): ConnectionScenario {
  return connectionFixtures[name];
}

export function getRandomConnectionFailure(): ConnectionScenario {
  const failureScenarios = Object.values(connectionFixtures).filter(
    scenario => !scenario.expectedResult.success
  );
  const randomIndex = Math.floor(Math.random() * failureScenarios.length);
  return failureScenarios[randomIndex];
}

export function getRandomConnectionSuccess(): ConnectionScenario {
  const successScenarios = Object.values(connectionFixtures).filter(
    scenario => scenario.expectedResult.success
  );
  const randomIndex = Math.floor(Math.random() * successScenarios.length);
  return successScenarios[randomIndex];
}

// Generate connection parameters with specific characteristics
export function generateConnectionParams(overrides?: Partial<ConnectParams>): ConnectParams {
  return {
    host: clusterHosts.mock,
    username: testUsers.valid,
    password: 'test_password_123',
    ...overrides,
  };
}

// Create session info with specific timestamps
export function createSessionInfo(minutesAgo: number, overrides?: Partial<SessionInfo>): SessionInfo {
  return {
    host: clusterHosts.mock,
    username: testUsers.valid,
    connectedAt: new Date(Date.now() - minutesAgo * 60 * 1000).toISOString(),
    ...overrides,
  };
}

// Validate connection parameters for testing input validation
export function validateConnectionParams(params: ConnectParams): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (!params.host?.trim()) {
    errors.push('Host is required');
  }
  
  if (!params.username?.trim()) {
    errors.push('Username is required');
  }
  
  if (!params.password?.trim()) {
    errors.push('Password is required');
  }
  
  return {
    valid: errors.length === 0,
    errors,
  };
}