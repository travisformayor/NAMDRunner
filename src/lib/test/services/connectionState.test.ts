import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ConnectionStateMachine } from '../../services/connectionState';
import type { ConnectionState, StateTransition } from '../../types/connection';
import { ErrorBuilder, CONNECTION_ERRORS } from '../../types/errors';

describe('ConnectionStateMachine', () => {
  let stateMachine: ConnectionStateMachine;

  beforeEach(() => {
    stateMachine = new ConnectionStateMachine();
  });

  describe('Initial State', () => {
    it('should start in Disconnected state', () => {
      expect(stateMachine.getCurrentState()).toBe('Disconnected');
    });

    it('should not be connected initially', () => {
      expect(stateMachine.isConnected()).toBe(false);
    });

    it('should be able to retry initially', () => {
      expect(stateMachine.canRetry()).toBe(true);
    });
  });

  describe('State Transitions', () => {
    it('should allow valid transitions from Disconnected', () => {
      const result = stateMachine.transitionTo('Connecting', 'User initiated connection');
      
      expect(result.success).toBe(true);
      expect(stateMachine.getCurrentState()).toBe('Connecting');
    });

    it('should reject invalid transitions', () => {
      const result = stateMachine.transitionTo('Connected', 'Invalid direct connection');
      
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      expect(stateMachine.getCurrentState()).toBe('Disconnected');
    });

    it('should support successful connection workflow', () => {
      // Normal connection flow
      expect(stateMachine.transitionTo('Connecting').success).toBe(true);
      expect(stateMachine.transitionTo('Connected').success).toBe(true);
      expect(stateMachine.transitionTo('Disconnected').success).toBe(true);
    });

    it('should support failed connection workflow', () => {
      // Failed connection flow
      expect(stateMachine.transitionTo('Connecting').success).toBe(true);
      expect(stateMachine.transitionTo('Disconnected').success).toBe(true); // Connection failed
    });

    it('should support session expiry workflow', () => {
      // Set up connected state
      stateMachine.transitionTo('Connecting');
      stateMachine.transitionTo('Connected');

      // Session expires, can reconnect
      expect(stateMachine.transitionTo('Expired').success).toBe(true);
      expect(stateMachine.transitionTo('Connecting').success).toBe(true);
    });

    it('should reject invalid direct transitions', () => {
      // Cannot jump directly to Connected
      expect(stateMachine.transitionTo('Connected').success).toBe(false);
      expect(stateMachine.getCurrentState()).toBe('Disconnected');

      // Cannot expire while disconnected
      expect(stateMachine.transitionTo('Expired').success).toBe(false);
    });
  });

  describe('Connection State Tracking', () => {
    it('should track connection state accurately', () => {
      expect(stateMachine.isConnected()).toBe(false);
      
      stateMachine.transitionTo('Connecting');
      expect(stateMachine.isConnected()).toBe(false);
      
      stateMachine.transitionTo('Connected');
      expect(stateMachine.isConnected()).toBe(true);
      
      stateMachine.transitionTo('Expired');
      expect(stateMachine.isConnected()).toBe(false);
    });

    it('should reset retry count on successful connection', () => {
      // Simulate failed connections
      stateMachine.setLastError(ErrorBuilder.create(CONNECTION_ERRORS.NETWORK_UNREACHABLE));
      stateMachine.setLastError(ErrorBuilder.create(CONNECTION_ERRORS.NETWORK_UNREACHABLE));
      
      expect(stateMachine.getRetryCount()).toBeGreaterThan(0);
      
      // Successful connection should reset count
      stateMachine.transitionTo('Connecting');
      stateMachine.transitionTo('Connected');
      
      expect(stateMachine.getRetryCount()).toBe(0);
      expect(stateMachine.getLastError()).toBeNull();
    });
  });

  describe('State History', () => {
    it('should record state transitions', () => {
      stateMachine.transitionTo('Connecting', 'User action');
      stateMachine.transitionTo('Connected', 'Auth success');
      
      const history = stateMachine.getStateHistory();
      
      expect(history).toHaveLength(3); // Initial + 2 transitions
      expect(history[1].from).toBe('Disconnected');
      expect(history[1].to).toBe('Connecting');
      expect(history[1].reason).toBe('User action');
      expect(history[2].from).toBe('Connecting');
      expect(history[2].to).toBe('Connected');
      expect(history[2].reason).toBe('Auth success');
    });

    it('should limit history size', () => {
      // Make many transitions to test history limit
      for (let i = 0; i < 100; i++) {
        stateMachine.forceState('Connecting', `Transition ${i}`);
        stateMachine.forceState('Disconnected', `Rollback ${i}`);
      }
      
      const history = stateMachine.getStateHistory();
      expect(history.length).toBeLessThanOrEqual(50);
    });
  });

  describe('Observer Pattern', () => {
    it('should notify observers of state changes', () => {
      const observer = vi.fn();
      const unsubscribe = stateMachine.onStateChange(observer);
      
      stateMachine.transitionTo('Connecting');
      stateMachine.transitionTo('Connected');
      
      expect(observer).toHaveBeenCalledTimes(2);
      expect(observer).toHaveBeenNthCalledWith(1, 'Connecting');
      expect(observer).toHaveBeenNthCalledWith(2, 'Connected');
      
      unsubscribe();
      
      stateMachine.transitionTo('Disconnected');
      expect(observer).toHaveBeenCalledTimes(2); // Should not be called after unsubscribe
    });

    it('should handle observer errors gracefully', () => {
      const faultyObserver = vi.fn().mockImplementation(() => {
        throw new Error('Observer error');
      });
      
      stateMachine.onStateChange(faultyObserver);
      
      // Should not throw error
      expect(() => {
        stateMachine.transitionTo('Connecting');
      }).not.toThrow();
    });
  });

  describe('Error Handling', () => {
    it('should track last error and retry count', () => {
      const error = ErrorBuilder.create(CONNECTION_ERRORS.NETWORK_UNREACHABLE);
      
      stateMachine.setLastError(error);
      
      expect(stateMachine.getLastError()).toBe(error);
      expect(stateMachine.getRetryCount()).toBe(1);
    });

    it('should respect retry limits', () => {
      // Simulate multiple network errors
      for (let i = 0; i < 5; i++) {
        stateMachine.setLastError(ErrorBuilder.create(CONNECTION_ERRORS.NETWORK_UNREACHABLE));
      }
      
      expect(stateMachine.canRetry()).toBe(false);
    });

    it('should allow retries for appropriate states', () => {
      stateMachine.forceState('Connected', 'Test setup');
      expect(stateMachine.canRetry()).toBe(false);
      
      stateMachine.forceState('Expired', 'Session expired');
      expect(stateMachine.canRetry()).toBe(true);
      
      stateMachine.forceState('Disconnected', 'Connection lost');
      expect(stateMachine.canRetry()).toBe(true);
    });
  });

  describe('Time-based Operations', () => {
    it('should detect idle connections based on threshold', () => {
      stateMachine.transitionTo('Connecting');
      stateMachine.transitionTo('Connected');
      
      // Should not be idle with large threshold
      expect(stateMachine.isIdleTooLong(1000000)).toBe(false);
      
      // Should not be idle when not connected
      stateMachine.transitionTo('Disconnected');
      expect(stateMachine.isIdleTooLong(0)).toBe(false);
    });
  });

  describe('Force State Changes', () => {
    it('should allow forcing state for error conditions', () => {
      stateMachine.transitionTo('Connecting');
      
      // Force to disconnected (simulating error)
      stateMachine.forceState('Disconnected', 'Network error');
      
      expect(stateMachine.getCurrentState()).toBe('Disconnected');
      
      const history = stateMachine.getStateHistory();
      const lastTransition = history[history.length - 1];
      expect(lastTransition.reason).toBe('FORCED: Network error');
    });
  });

  describe('Diagnostics', () => {
    it('should provide comprehensive diagnostics', () => {
      stateMachine.transitionTo('Connecting');
      stateMachine.transitionTo('Connected');
      
      const error = ErrorBuilder.create(CONNECTION_ERRORS.NETWORK_UNREACHABLE);
      stateMachine.setLastError(error);
      
      const diagnostics = stateMachine.getDiagnostics();
      
      expect(diagnostics.currentState).toBe('Connected');
      expect(diagnostics.retryCount).toBe(1);
      expect(diagnostics.observerCount).toBe(0);
      expect(diagnostics.historyLength).toBeGreaterThan(0);
      expect(diagnostics.lastError).toBe(error);
      expect(diagnostics.validTransitions).toContain('Disconnected');
      expect(diagnostics.validTransitions).toContain('Expired');
    });
  });

  describe('Custom Initial State', () => {
    it('should accept custom initial state', () => {
      const customStateMachine = new ConnectionStateMachine('Connected');
      
      expect(customStateMachine.getCurrentState()).toBe('Connected');
      expect(customStateMachine.isConnected()).toBe(true);
    });
  });
});