import type { 
  ConnectionState, 
  ConnectionStateManager, 
  StateTransition, 
  Result 
} from '../types/connection';
import type { ConnectionError } from '../types/errors';
import { ErrorBuilder } from '../types/errors';
import { toConnectionError } from '../types/errorUtils';

// Valid state transitions map
const VALID_TRANSITIONS: Record<ConnectionState, ConnectionState[]> = {
  'Disconnected': ['Connecting'],
  'Connecting': ['Connected', 'Disconnected'],
  'Connected': ['Disconnected', 'Expired'],
  'Expired': ['Connecting', 'Disconnected']
};

/**
 * Connection state manager with observable state transitions
 * Manages connection lifecycle and validates state changes
 */
export class ConnectionStateMachine implements ConnectionStateManager {
  private currentState: ConnectionState = 'Disconnected';
  private stateHistory: StateTransition[] = [];
  private observers: Array<(state: ConnectionState) => void> = [];
  private lastError: ConnectionError | null = null;
  private retryCount = 0;
  private maxRetries = 3;

  constructor(initialState: ConnectionState = 'Disconnected') {
    this.currentState = initialState;
    this.recordTransition('Disconnected', initialState, 'Initial state');
  }

  getCurrentState(): ConnectionState {
    return this.currentState;
  }

  transitionTo(newState: ConnectionState, reason?: string): Result<void> {
    if (!this.canTransition(this.currentState, newState)) {
      const error = ErrorBuilder.create({
        code: 'STATE_001',
        category: 'Validation',
        message: `Invalid state transition from ${this.currentState} to ${newState}`,
        retryable: false,
        suggestions: [
          `Current state: ${this.currentState}`,
          `Valid transitions: ${VALID_TRANSITIONS[this.currentState].join(', ')}`,
          'Check the connection state before attempting this operation'
        ]
      });

      return {
        success: false,
        error
      };
    }

    const previousState = this.currentState;
    this.currentState = newState;
    
    this.recordTransition(previousState, newState, reason);
    this.notifyObservers(newState);

    // Reset retry count on successful connection
    if (newState === 'Connected') {
      this.retryCount = 0;
      this.lastError = null;
    }

    return { success: true, data: undefined };
  }

  canTransition(from: ConnectionState, to: ConnectionState): boolean {
    return VALID_TRANSITIONS[from].includes(to);
  }

  isConnected(): boolean {
    return this.currentState === 'Connected';
  }

  canRetry(): boolean {
    return this.retryCount < this.maxRetries && 
           (this.currentState === 'Disconnected' || this.currentState === 'Expired');
  }

  getStateHistory(): StateTransition[] {
    return [...this.stateHistory];
  }

  onStateChange(callback: (state: ConnectionState) => void): () => void {
    this.observers.push(callback);
    
    // Return unsubscribe function
    return () => {
      const index = this.observers.indexOf(callback);
      if (index > -1) {
        this.observers.splice(index, 1);
      }
    };
  }

  // Additional methods for error handling
  setLastError(error: ConnectionError): void {
    this.lastError = error;
    if (error.category === 'Network' || error.category === 'Timeout') {
      this.retryCount++;
    }
  }

  getLastError(): ConnectionError | null {
    return this.lastError;
  }

  getRetryCount(): number {
    return this.retryCount;
  }

  resetRetryCount(): void {
    this.retryCount = 0;
  }

  // Force state change (for error conditions)
  forceState(newState: ConnectionState, reason: string): void {
    const previousState = this.currentState;
    this.currentState = newState;
    this.recordTransition(previousState, newState, `FORCED: ${reason}`);
    this.notifyObservers(newState);
  }

  // Get time since last state change
  getTimeSinceLastTransition(): number {
    if (this.stateHistory.length === 0) return 0;
    
    const lastTransition = this.stateHistory[this.stateHistory.length - 1];
    return Date.now() - new Date(lastTransition.timestamp).getTime();
  }

  // Check if connection has been idle too long
  isIdleTooLong(maxIdleMs: number = 30 * 60 * 1000): boolean { // 30 minutes default
    return this.currentState === 'Connected' && 
           this.getTimeSinceLastTransition() > maxIdleMs;
  }

  private recordTransition(
    from: ConnectionState, 
    to: ConnectionState, 
    reason?: string
  ): void {
    const transition: StateTransition = {
      from,
      to,
      timestamp: new Date().toISOString(),
      reason,
      success: true
    };

    this.stateHistory.push(transition);
    
    // Keep only last 50 transitions to prevent memory leaks
    if (this.stateHistory.length > 50) {
      this.stateHistory = this.stateHistory.slice(-50);
    }
  }

  private notifyObservers(state: ConnectionState): void {
    this.observers.forEach(observer => {
      try {
        observer(state);
      } catch (error) {
        console.error('Error in state change observer:', error);
      }
    });
  }

  // Diagnostic methods for debugging
  getDiagnostics() {
    return {
      currentState: this.currentState,
      retryCount: this.retryCount,
      maxRetries: this.maxRetries,
      observerCount: this.observers.length,
      historyLength: this.stateHistory.length,
      lastTransition: this.stateHistory[this.stateHistory.length - 1],
      timeSinceLastTransition: this.getTimeSinceLastTransition(),
      lastError: this.lastError,
      validTransitions: VALID_TRANSITIONS[this.currentState]
    };
  }
}

// Factory function for creating state managers
export function createConnectionStateManager(
  initialState: ConnectionState = 'Disconnected'
): ConnectionStateManager {
  return new ConnectionStateMachine(initialState);
}