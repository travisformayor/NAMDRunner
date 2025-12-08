import { describe, it, expect } from 'vitest';
import { isConnectionError } from './storeFactory';

describe('Connection Error Detection', () => {

  it('should detect timeout errors', () => {
    expect(isConnectionError('Connection timeout')).toBe(true);
    expect(isConnectionError('Request timed out')).toBe(true);
    expect(isConnectionError('SSH timeout after 30s')).toBe(true);
  });

  it('should detect not connected errors', () => {
    expect(isConnectionError('Not connected to cluster')).toBe(true);
    expect(isConnectionError('not connected')).toBe(true);
    expect(isConnectionError('Connection not established')).toBe(true);
  });

  it('should detect connection failure errors', () => {
    expect(isConnectionError('Connection failed')).toBe(true);
    expect(isConnectionError('Connection refused')).toBe(true);
    expect(isConnectionError('Connection reset')).toBe(true);
  });

  it('should detect broken pipe errors', () => {
    expect(isConnectionError('Broken pipe')).toBe(true);
    expect(isConnectionError('broken pipe detected')).toBe(true);
  });

  it('should detect network errors', () => {
    expect(isConnectionError('Network unreachable')).toBe(true);
    expect(isConnectionError('Network error occurred')).toBe(true);
  });

  it('should detect SSH errors', () => {
    expect(isConnectionError('SSH connection lost')).toBe(true);
    expect(isConnectionError('ssh handshake failed')).toBe(true);
  });

  it('should be case insensitive', () => {
    expect(isConnectionError('CONNECTION TIMEOUT')).toBe(true);
    expect(isConnectionError('Network Error')).toBe(true);
    expect(isConnectionError('SSH Failed')).toBe(true);
  });

  it('should not detect unrelated errors', () => {
    expect(isConnectionError('Template not found')).toBe(false);
    expect(isConnectionError('Invalid job ID')).toBe(false);
    expect(isConnectionError('Database error')).toBe(false);
    expect(isConnectionError('Validation failed')).toBe(false);
    expect(isConnectionError('File upload failed')).toBe(false);
  });

  it('should handle empty strings', () => {
    expect(isConnectionError('')).toBe(false);
  });

  it('should match keywords anywhere in message', () => {
    // Matches if keyword appears anywhere
    expect(isConnectionError('disconnected from server')).toBe(true);
    expect(isConnectionError('disk connection full')).toBe(true);  // "connection" keyword present
    expect(isConnectionError('network interface error')).toBe(true); // "network" keyword present
  });
});
