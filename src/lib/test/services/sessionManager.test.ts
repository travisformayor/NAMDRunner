import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { SSHSessionManager, SessionValidation } from '../../services/sessionManager';
import type { SessionInfo } from '../../types/api';
import { CONNECTION_ERRORS } from '../../types/errors';

describe('SSHSessionManager', () => {
  let sessionManager: SSHSessionManager;
  let mockSessionInfo: SessionInfo;

  beforeEach(() => {
    sessionManager = new SSHSessionManager();
    mockSessionInfo = {
      host: 'test.cluster.com',
      username: 'testuser',
      connectedAt: new Date().toISOString()
    };
  });

  afterEach(() => {
    sessionManager.dispose();
  });

  describe('Session Persistence', () => {
    it('should save and load valid session', async () => {
      const saveResult = await sessionManager.saveSession(mockSessionInfo);
      expect(saveResult.success).toBe(true);

      const loadResult = await sessionManager.loadSession();
      expect(loadResult.success).toBe(true);
      expect(loadResult.data).toEqual(mockSessionInfo);
    });

    it('should reject invalid session info', async () => {
      const invalidSession = {
        host: '',
        username: 'testuser',
        connectedAt: new Date().toISOString()
      } as SessionInfo;

      const result = await sessionManager.saveSession(invalidSession);
      expect(result.success).toBe(false);
      expect(result.error?.message).toContain('Invalid session information');
    });

    it('should return null when no session exists', async () => {
      const result = await sessionManager.loadSession();
      expect(result.success).toBe(true);
      expect(result.data).toBeNull();
    });

    it('should clear session completely', async () => {
      await sessionManager.saveSession(mockSessionInfo);
      
      const clearResult = await sessionManager.clearSession();
      expect(clearResult.success).toBe(true);

      const loadResult = await sessionManager.loadSession();
      expect(loadResult.data).toBeNull();
    });
  });

  describe('Session Validation', () => {
    it('should validate fresh sessions', () => {
      const freshSession: SessionInfo = {
        host: 'test.cluster.com',
        username: 'testuser',
        connectedAt: new Date().toISOString()
      };

      expect(sessionManager.isSessionValid(freshSession)).toBe(true);
    });

    it('should invalidate expired sessions', () => {
      const expiredSession: SessionInfo = {
        host: 'test.cluster.com',
        username: 'testuser',
        connectedAt: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString() // 5 hours ago
      };

      expect(sessionManager.isSessionValid(expiredSession)).toBe(false);
    });

    it('should invalidate sessions with missing data', () => {
      expect(sessionManager.isSessionValid(null as any)).toBe(false);
      expect(sessionManager.isSessionValid({} as SessionInfo)).toBe(false);
      
      const incompleteSession = {
        host: 'test.cluster.com',
        username: 'testuser'
        // Missing connectedAt
      } as SessionInfo;
      
      expect(sessionManager.isSessionValid(incompleteSession)).toBe(false);
    });

    it('should return expired error for invalid loaded session', async () => {
      const expiredSession: SessionInfo = {
        host: 'test.cluster.com',
        username: 'testuser',
        connectedAt: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString()
      };

      await sessionManager.saveSession(expiredSession);
      
      const loadResult = await sessionManager.loadSession();
      expect(loadResult.success).toBe(false);
      expect(loadResult.error?.code).toBe(CONNECTION_ERRORS.SESSION_EXPIRED.code);
    });
  });

  describe('Session Refresh', () => {
    it('should refresh session with callback', async () => {
      const refreshCallback = vi.fn().mockResolvedValue({
        host: 'test.cluster.com',
        username: 'testuser',
        connectedAt: new Date().toISOString()
      });

      const managerWithRefresh = new SSHSessionManager(4 * 60 * 60 * 1000, refreshCallback);
      
      await managerWithRefresh.saveSession(mockSessionInfo);
      
      const refreshResult = await managerWithRefresh.refreshSession();
      expect(refreshResult.success).toBe(true);
      expect(refreshCallback).toHaveBeenCalled();
      
      managerWithRefresh.dispose();
    });

    it('should fail refresh without callback', async () => {
      await sessionManager.saveSession(mockSessionInfo);
      
      const refreshResult = await sessionManager.refreshSession();
      expect(refreshResult.success).toBe(false);
      expect(refreshResult.error?.message).toContain('No refresh callback configured');
    });

    it('should fail refresh without active session', async () => {
      const refreshCallback = vi.fn();
      const managerWithRefresh = new SSHSessionManager(4 * 60 * 60 * 1000, refreshCallback);
      
      const refreshResult = await managerWithRefresh.refreshSession();
      expect(refreshResult.success).toBe(false);
      expect(refreshResult.error?.message).toContain('No active session to refresh');
      
      managerWithRefresh.dispose();
    });

    it('should handle refresh callback errors', async () => {
      const refreshCallback = vi.fn().mockRejectedValue(new Error('Refresh failed'));
      const managerWithRefresh = new SSHSessionManager(4 * 60 * 60 * 1000, refreshCallback);
      
      await managerWithRefresh.saveSession(mockSessionInfo);
      
      const refreshResult = await managerWithRefresh.refreshSession();
      expect(refreshResult.success).toBe(false);
      expect(refreshResult.error?.message).toContain('Refresh failed');
      
      managerWithRefresh.dispose();
    });
  });

  describe('Automatic Refresh Scheduling', () => {
    it('should schedule automatic refresh', async () => {
      vi.useFakeTimers();
      
      const refreshCallback = vi.fn().mockResolvedValue({
        host: 'test.cluster.com',
        username: 'testuser',
        connectedAt: new Date().toISOString()
      });

      const managerWithRefresh = new SSHSessionManager(1000, refreshCallback); // 1 second validity
      
      await managerWithRefresh.saveSession(mockSessionInfo);
      managerWithRefresh.scheduleSessionRefresh(500); // Refresh every 500ms
      
      // Fast forward time
      vi.advanceTimersByTime(600);
      
      await vi.runAllTimersAsync();
      
      expect(refreshCallback).toHaveBeenCalled();
      
      managerWithRefresh.dispose();
      vi.useRealTimers();
    });

    it('should cancel scheduled refresh', async () => {
      vi.useFakeTimers();
      
      const refreshCallback = vi.fn();
      const managerWithRefresh = new SSHSessionManager(1000, refreshCallback);
      
      await managerWithRefresh.saveSession(mockSessionInfo);
      managerWithRefresh.scheduleSessionRefresh(500);
      managerWithRefresh.cancelSessionRefresh();
      
      vi.advanceTimersByTime(600);
      await vi.runAllTimersAsync();
      
      expect(refreshCallback).not.toHaveBeenCalled();
      
      managerWithRefresh.dispose();
      vi.useRealTimers();
    });
  });

  describe('Session Age and Diagnostics', () => {
    it('should track session age and expiry', async () => {
      const oneMinuteAgo = new Date(Date.now() - 60 * 1000).toISOString();
      const sessionOneMinuteAgo = { ...mockSessionInfo, connectedAt: oneMinuteAgo };

      await sessionManager.saveSession(sessionOneMinuteAgo);

      // Check age calculation
      const age = sessionManager.getSessionAge();
      expect(age).toBeGreaterThan(59000);

      // Check expiry detection
      const timeUntilExpiry = sessionManager.getTimeUntilExpiry();
      expect(timeUntilExpiry).toBeGreaterThan(0);

      // Check diagnostics
      const diagnostics = sessionManager.getSessionDiagnostics();
      expect(diagnostics.hasSession).toBe(true);
      expect(diagnostics.isValid).toBe(true);
    });

    it('should handle no session state', () => {
      expect(sessionManager.getSessionAge()).toBeNull();
      expect(sessionManager.getSessionInfo()).toBeNull();

      const diagnostics = sessionManager.getSessionDiagnostics();
      expect(diagnostics.hasSession).toBe(false);
      expect(diagnostics.isValid).toBe(false);
    });

    it('should detect expiring sessions', async () => {
      const almostExpiredSession = {
        ...mockSessionInfo,
        connectedAt: new Date(Date.now() - 3.5 * 60 * 60 * 1000).toISOString() // 3.5 hours ago
      };

      await sessionManager.saveSession(almostExpiredSession);
      expect(sessionManager.isExpiringSoon()).toBe(true);
    });
  });

  describe('Cleanup and Disposal', () => {
    it('should clean up resources on dispose', () => {
      const managerWithRefresh = new SSHSessionManager(1000, vi.fn());
      managerWithRefresh.scheduleSessionRefresh(500);
      
      // Should not throw
      expect(() => managerWithRefresh.dispose()).not.toThrow();
    });
  });
});

describe('SessionValidation Utilities', () => {
  const validSession: SessionInfo = {
    host: 'test.cluster.com',
    username: 'testuser',
    connectedAt: new Date().toISOString()
  };

  it('should validate complete vs incomplete session info', () => {
    // Valid session
    expect(SessionValidation.hasRequiredFields(validSession)).toBe(true);

    // Invalid sessions
    expect(SessionValidation.hasRequiredFields({})).toBe(false);
    expect(SessionValidation.hasRequiredFields({ host: 'test.com' })).toBe(false);
  });

  it('should sanitize session for logging', () => {
    const sanitized = SessionValidation.sanitizeForLogging(validSession);
    expect(Object.keys(sanitized)).toEqual(['host', 'username', 'connectedAt']);
  });

  it('should format session age correctly', () => {
    const oneMinuteAgo = new Date(Date.now() - 60 * 1000).toISOString();
    const session = { ...validSession, connectedAt: oneMinuteAgo };

    expect(SessionValidation.getSessionAgeMinutes(session)).toBe(1);
    expect(SessionValidation.formatSessionAge(session)).toBe('1m ago');
  });
});