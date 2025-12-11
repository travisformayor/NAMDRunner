import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST } as any)],
  test: {
    environment: 'node',
    globals: true,
    setupFiles: ['./src/lib/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,ts}'],
    // Timeout configuration
    testTimeout: 10000,
    hookTimeout: 10000,
    // Use threads pool for faster execution
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: true,
      },
    },
    // Disable coverage to speed up tests
    coverage: {
      enabled: false,
    },
    // Disable watch mode features for faster startup
    watch: false,
    // Cache test results
    cache: {
      dir: 'node_modules/.vitest',
    },
  },
  resolve: {
    alias: {
      '$lib': '/src/lib',
      '$app': '/node_modules/@sveltejs/kit/src/runtime/app',
    },
  },
});