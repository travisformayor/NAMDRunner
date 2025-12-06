/**
 * Cluster Configuration Store
 *
 * Single source of truth for cluster capabilities (partitions, QOS, presets, billing).
 * Backend owns the data, frontend caches it here for fast UI access.
 *
 * Architecture:
 * - Backend (cluster_config.rs) is the authoritative source
 * - Load once on app init via get_cluster_capabilities() IPC command
 * - Components subscribe to this store for instant access
 * - All validation done via backend validate_resource_request() command
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  ClusterCapabilities,
  PartitionSpec,
  QosSpec,
  ValidationResult,
  ApiResult
} from '../types/api';

// Main store - holds full cluster capabilities from backend
const clusterCapabilitiesStore = writable<ClusterCapabilities | null>(null);
const isLoadedStore = writable<boolean>(false);
const loadErrorStore = writable<string | null>(null);

/**
 * Initialize cluster configuration - call once on app startup
 */
export async function initClusterConfig(): Promise<void> {
  try {
    loadErrorStore.set(null);
    const result = await invoke<ApiResult<ClusterCapabilities>>('get_cluster_capabilities');

    if (result.success && result.data) {
      clusterCapabilitiesStore.set(result.data);
      isLoadedStore.set(true);
    } else {
      const error = result.error || 'Failed to load cluster configuration';
      loadErrorStore.set(error);
      throw new Error(error);
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : 'Unknown error loading cluster config';
    loadErrorStore.set(errorMsg);
    throw error;
  }
}


// Derived store: all partitions (for dropdowns)
export const partitions = derived(
  clusterCapabilitiesStore,
  $config => $config?.partitions ?? []
);

// Derived store: all QOS options (all partitions)
export const allQosOptions = derived(
  clusterCapabilitiesStore,
  $config => $config?.qos_options ?? []
);

// Derived store: job presets
export const jobPresets = derived(
  clusterCapabilitiesStore,
  $config => $config?.job_presets ?? []
);

// Derived store: billing rates
export const billingRates = derived(
  clusterCapabilitiesStore,
  $config => $config?.billing_rates
);


/**
 * Calculate estimated job cost via backend
 * Backend is the single source of truth for billing rates and calculations
 */
export async function calculateJobCost(
  cores: number,
  walltimeHours: number,
  hasGpu: boolean = false,
  gpuCount: number = 1
): Promise<number> {
  try {
    return await invoke<number>('calculate_job_cost', {
      cores,
      walltime_hours: walltimeHours,
      has_gpu: hasGpu,
      gpu_count: gpuCount
    });
  } catch (error) {
    return 0;
  }
}

/**
 * Convert walltime string (HH:MM:SS) to hours
 */
export function walltimeToHours(walltime: string): number {
  if (!walltime) return 0;
  const parts = walltime.split(':');
  if (parts.length !== 3) return 0;

  const hours = parseInt(parts[0] || '0') || 0;
  const minutes = parseInt(parts[1] || '0') || 0;
  const seconds = parseInt(parts[2] || '0') || 0;

  return hours + minutes / 60 + seconds / 3600;
}


/**
 * Estimate queue time based on resources and partition via backend
 * Backend is the single source of truth for queue time heuristics
 */
export async function estimateQueueTime(cores: number, partitionId: string): Promise<string> {
  try {
    return await invoke<string>('estimate_queue_time_for_job', {
      cores,
      partition_id: partitionId
    });
  } catch (error) {
    return 'Unknown';
  }
}

/**
 * Validate resource request via backend
 * Calls the backend validate_resource_allocation command for comprehensive validation
 *
 * All parameters are passed directly to backend - no frontend parsing or conversion.
 * Backend is the single source of truth for validation logic.
 */
export async function validateResourceRequest(
  cores: number,
  memory: string,
  walltime: string,
  partitionId: string,
  qosId: string
): Promise<ValidationResult> {
  // Check if config is loaded
  const config = get(clusterCapabilitiesStore);
  if (!config) {
    return {
      is_valid: false,
      issues: ['Cluster configuration not loaded'],
      warnings: [],
      suggestions: []
    };
  }

  // Call backend validation - pass parameters as-is, no conversion
  try {
    const result = await invoke<ValidationResult>('validate_resource_allocation', {
      cores,
      memory,
      walltime,
      partition_id: partitionId,
      qos_id: qosId
    });
    return result;
  } catch (error) {
    return {
      is_valid: false,
      issues: ['Validation failed: ' + (error instanceof Error ? error.message : 'Unknown error')],
      warnings: [],
      suggestions: []
    };
  }
}

/**
 * Set cluster capabilities directly (used by centralized app initialization)
 */
export function setClusterCapabilities(capabilities: ClusterCapabilities): void {
  clusterCapabilitiesStore.set(capabilities);
  isLoadedStore.set(true);
  loadErrorStore.set(null);
}

// Export main store and utilities
export const clusterConfig = {
  subscribe: clusterCapabilitiesStore.subscribe,
  init: initClusterConfig,
  refresh: initClusterConfig,
  set: setClusterCapabilities,
};

