import { vi } from 'vitest';

// Mock Tauri APIs for testing
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Make mock available globally for tests
(globalThis as any).__TAURI_INVOKE__ = mockInvoke;

// Reset mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
});