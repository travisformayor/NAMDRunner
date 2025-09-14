import type { SessionInfo } from '../types/api';
import type { SessionManager, Result } from '../types/connection';
import type { ConnectionError } from '../types/errors';
import { ErrorBuilder, CONNECTION_ERRORS } from '../types/errors';
import { toConnectionError, wrapWithResult } from '../types/errorUtils';

/**
 * Session manager for handling SSH session lifecycle
 * Manages session persistence, validation, and cleanup
 * SECURITY: No credentials are persisted - only session metadata
 */
export class SSHSessionManager implements SessionManager {
  private currentSession: SessionInfo | null = null;
  private refreshTimer: NodeJS.Timeout | null = null;
  private sessionValidityMs: number;
  private refreshCallback?: () => Promise<SessionInfo>;

  constructor(
    sessionValidityMs: number = 4 * 60 * 60 * 1000, // 4 hours default
    refreshCallback?: () => Promise<SessionInfo>
  ) {
    this.sessionValidityMs = sessionValidityMs;
    this.refreshCallback = refreshCallback;
  }

  async saveSession(sessionInfo: SessionInfo): Promise<Result<void>> {
    try {
      // Validate session info
      if (!sessionInfo.host || !sessionInfo.username || !sessionInfo.connectedAt) {
        return {
          success: false,
          error: ErrorBuilder.create(CONNECTION_ERRORS.INVALID_CONFIG, 'Invalid session information')
        };
      }

      this.currentSession = {
        ...sessionInfo,
        connectedAt: sessionInfo.connectedAt || new Date().toISOString()
      };

      // Schedule automatic refresh if callback provided
      this.scheduleSessionRefresh();

      return { success: true, data: undefined };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Configuration')
      };
    }
  }

  async loadSession(): Promise<Result<SessionInfo | null>> {
    try {
      if (!this.currentSession) {
        return { success: true, data: null };
      }

      // Check if session is still valid
      if (!this.isSessionValid(this.currentSession)) {
        await this.clearSession();
        return {
          success: false,
          error: ErrorBuilder.create(CONNECTION_ERRORS.SESSION_EXPIRED)
        };
      }

      return { success: true, data: this.currentSession };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Configuration')
      };
    }
  }

  isSessionValid(sessionInfo: SessionInfo): boolean {
    if (!sessionInfo || !sessionInfo.connectedAt) {
      return false;
    }

    const sessionAge = Date.now() - new Date(sessionInfo.connectedAt).getTime();
    return sessionAge < this.sessionValidityMs;
  }

  async clearSession(): Promise<Result<void>> {
    try {
      this.currentSession = null;
      this.cancelSessionRefresh();
      
      // Clear any sensitive data from memory
      if (global.gc) {
        global.gc();
      }

      return { success: true, data: undefined };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Unknown')
      };
    }
  }

  async refreshSession(): Promise<Result<SessionInfo>> {
    try {
      if (!this.refreshCallback) {
        return {
          success: false,
          error: ErrorBuilder.create(CONNECTION_ERRORS.INVALID_CONFIG, 'No refresh callback configured')
        };
      }

      if (!this.currentSession) {
        return {
          success: false,
          error: ErrorBuilder.create(CONNECTION_ERRORS.SESSION_EXPIRED, 'No active session to refresh')
        };
      }

      const refreshedSession = await this.refreshCallback();
      const result = await this.saveSession(refreshedSession);
      
      if (!result.success) {
        return {
          success: false,
          error: result.error!
        };
      }

      return { success: true, data: refreshedSession };
    } catch (error) {
      return {
        success: false,
        error: ErrorBuilder.fromError(error as Error, 'Network')
      };
    }
  }

  scheduleSessionRefresh(intervalMs?: number): void {
    // Clear existing timer
    this.cancelSessionRefresh();

    if (!this.refreshCallback) {
      return;
    }

    const refreshInterval = intervalMs || (this.sessionValidityMs * 0.8); // Refresh at 80% of validity period
    
    this.refreshTimer = setTimeout(async () => {
      try {
        const result = await this.refreshSession();
        if (!result.success) {
          console.warn('Automatic session refresh failed:', result.error);
        }
        
        // Schedule next refresh
        this.scheduleSessionRefresh(intervalMs);
      } catch (error) {
        console.error('Error during automatic session refresh:', error);
      }
    }, refreshInterval);
  }

  cancelSessionRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  getSessionAge(): number | null {
    if (!this.currentSession || !this.currentSession.connectedAt) {
      return null;
    }

    return Date.now() - new Date(this.currentSession.connectedAt).getTime();
  }

  // Additional utility methods
  getTimeUntilExpiry(): number | null {
    const age = this.getSessionAge();
    if (age === null) return null;
    
    return Math.max(0, this.sessionValidityMs - age);
  }

  isExpiringSoon(warningThresholdMs: number = 10 * 60 * 1000): boolean { // 10 minutes default
    const timeUntilExpiry = this.getTimeUntilExpiry();
    return timeUntilExpiry !== null && timeUntilExpiry < warningThresholdMs;
  }

  getSessionInfo(): SessionInfo | null {
    return this.currentSession ? { ...this.currentSession } : null;
  }

  // Session diagnostics
  getSessionDiagnostics() {
    const session = this.currentSession;
    if (!session) {
      return {
        hasSession: false,
        isValid: false,
        age: null,
        timeUntilExpiry: null,
        isExpiringSoon: false,
        refreshScheduled: this.refreshTimer !== null
      };
    }

    const age = this.getSessionAge();
    const timeUntilExpiry = this.getTimeUntilExpiry();
    
    return {
      hasSession: true,
      isValid: this.isSessionValid(session),
      age,
      timeUntilExpiry,
      isExpiringSoon: this.isExpiringSoon(),
      refreshScheduled: this.refreshTimer !== null,
      host: session.host,
      username: session.username,
      connectedAt: session.connectedAt
    };
  }

  // Cleanup method for proper disposal
  dispose(): void {
    this.cancelSessionRefresh();
    this.currentSession = null;
    this.refreshCallback = undefined;
  }
}

// Factory function for creating session managers
export function createSessionManager(
  sessionValidityMs?: number,
  refreshCallback?: () => Promise<SessionInfo>
): SessionManager {
  return new SSHSessionManager(sessionValidityMs, refreshCallback);
}

// Session validator utility functions
export const SessionValidation = {
  /**
   * Check if session info has required fields
   */
  hasRequiredFields(sessionInfo: Partial<SessionInfo>): sessionInfo is SessionInfo {
    return !!(sessionInfo.host && sessionInfo.username && sessionInfo.connectedAt);
  },

  /**
   * Sanitize session info for logging (remove sensitive data)
   */
  sanitizeForLogging(sessionInfo: SessionInfo): Partial<SessionInfo> {
    return {
      host: sessionInfo.host,
      username: sessionInfo.username,
      connectedAt: sessionInfo.connectedAt
    };
  },

  /**
   * Calculate session age in minutes
   */
  getSessionAgeMinutes(sessionInfo: SessionInfo): number {
    const age = Date.now() - new Date(sessionInfo.connectedAt).getTime();
    return Math.floor(age / (1000 * 60));
  },

  /**
   * Format session age for display
   */
  formatSessionAge(sessionInfo: SessionInfo): string {
    const ageMs = Date.now() - new Date(sessionInfo.connectedAt).getTime();
    const ageMinutes = Math.floor(ageMs / (1000 * 60));
    
    if (ageMinutes < 1) return 'Just connected';
    if (ageMinutes < 60) return `${ageMinutes}m ago`;
    
    const ageHours = Math.floor(ageMinutes / 60);
    if (ageHours < 24) return `${ageHours}h ago`;
    
    const ageDays = Math.floor(ageHours / 24);
    return `${ageDays}d ago`;
  }
};