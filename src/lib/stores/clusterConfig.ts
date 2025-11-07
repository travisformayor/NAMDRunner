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
import type { ClusterCapabilities, PartitionSpec, QosSpec, ValidationResult } from '../types/cluster';
import type { GetClusterCapabilitiesResult } from '../types/api';
import { CoreClientFactory } from '../ports/clientFactory';
import { logger } from '../utils/logger';

// Main store - holds full cluster capabilities from backend
const clusterCapabilitiesStore = writable<ClusterCapabilities | null>(null);
const isLoadedStore = writable<boolean>(false);
const loadErrorStore = writable<string | null>(null);

/**
 * Initialize cluster configuration - call once on app startup
 */
export async function initClusterConfig(): Promise<void> {
  logger.debug('ClusterConfig', 'Init started');
  try {
    loadErrorStore.set(null);
    logger.debug('ClusterConfig', 'Calling backend getClusterCapabilities...');
    const result: GetClusterCapabilitiesResult = await CoreClientFactory.getClient().getClusterCapabilities();
    logger.debug('ClusterConfig', `Backend response: success=${result.success}, hasData=${!!result.data}`);

    if (result.success && result.data) {
      logger.debug('ClusterConfig', `Setting cluster capabilities: ${result.data.partitions.length} partitions, ${result.data.qos_options.length} QOS, ${result.data.job_presets.length} presets`);
      clusterCapabilitiesStore.set(result.data);
      isLoadedStore.set(true);
      logger.debug('ClusterConfig', 'Cluster config loaded successfully');
    } else {
      const error = result.error || 'Failed to load cluster configuration';
      logger.debug('ClusterConfig', `Backend returned error: ${error}`);
      loadErrorStore.set(error);
      throw new Error(error);
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : 'Unknown error loading cluster config';
    logger.debug('ClusterConfig', `Exception during init: ${errorMsg}`);
    loadErrorStore.set(errorMsg);
    throw error;
  }
}

/**
 * Refresh cluster configuration from backend
 * Useful if config changes (future user-editable settings)
 */
export async function refreshClusterConfig(): Promise<void> {
  return initClusterConfig();
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
 * Get QOS options valid for a specific partition (reactive store)
 */
export function getQosForPartitionStore(partitionId: string) {
  return derived(
    clusterCapabilitiesStore,
    $config => {
      if (!$config) return [];
      return $config.qos_options.filter(qos =>
        qos.valid_partitions.includes(partitionId)
      );
}
  );
}

/**
 * Get QOS options valid for a specific partition (non-reactive)
 */
export function getQosForPartition(partitionId: string): QosSpec[] {
  const config = get(clusterCapabilitiesStore);
  if (!config) return [];
  return config.qos_options.filter(qos =>
    qos.valid_partitions.includes(partitionId)
  );
}

/**
 * Get a specific partition by ID
 */
export function getPartition(partitionId: string): PartitionSpec | null {
  const config = get(clusterCapabilitiesStore);
  if (!config) return null;
  return config.partitions.find(p => p.id === partitionId) ?? null;
}

/**
 * Get all partitions (non-reactive helper)
 */
export function getAllPartitions(): PartitionSpec[] {
  const config = get(clusterCapabilitiesStore);
  return config?.partitions ?? [];
}

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
    return await CoreClientFactory.getClient().calculateJobCost(cores, walltimeHours, hasGpu, gpuCount);
  } catch (error) {
    logger.debug('ClusterConfig', `Cost calculation failed: ${error}`);
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
 * Suggest optimal QOS based on walltime and partition via backend
 * Backend is the single source of truth for QoS selection rules
 */
export async function suggestQos(walltimeHours: number, partitionId: string): Promise<string> {
  try {
    return await CoreClientFactory.getClient().suggestQosForPartition(walltimeHours, partitionId);
  } catch (error) {
    logger.debug('ClusterConfig', `QoS suggestion failed: ${error}`);
    // Fallback to default QoS
    const config = get(clusterCapabilitiesStore);
    const validQos = getQosForPartition(partitionId);
    const defaultQos = validQos.find(q => q.is_default);
    return defaultQos?.id || validQos[0]?.id || (partitionId ? 'normal' : 'normal');
  }
}

/**
 * Estimate queue time based on resources and partition via backend
 * Backend is the single source of truth for queue time heuristics
 */
export async function estimateQueueTime(cores: number, partitionId: string): Promise<string> {
  try {
    return await CoreClientFactory.getClient().estimateQueueTimeForJob(cores, partitionId);
  } catch (error) {
    logger.debug('ClusterConfig', `Queue time estimation failed: ${error}`);
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
    logger.debug('ClusterConfig', `Calling backend validation: cores=${cores}, memory=${memory}, walltime=${walltime}, partition=${partitionId}, qos=${qosId}`);
    const result = await CoreClientFactory.getClient().validateResourceAllocation(
      cores,
      memory,
      walltime,
      partitionId,
      qosId
    );
    logger.debug('ClusterConfig', `Backend validation result: is_valid=${result.is_valid}, issues=${JSON.stringify(result.issues)}`);
    return result;
  } catch (error) {
    logger.debug('ClusterConfig', `Backend validation ERROR: ${error instanceof Error ? error.message : JSON.stringify(error)}`);
    return {
      is_valid: false,
      issues: ['Validation failed: ' + (error instanceof Error ? error.message : 'Unknown error')],
      warnings: [],
      suggestions: []
    };
  }
}

// Export main store and utilities
export const clusterConfig = {
  subscribe: clusterCapabilitiesStore.subscribe,
  init: initClusterConfig,
  refresh: refreshClusterConfig
};

export const isLoaded = {
  subscribe: isLoadedStore.subscribe
};

export const loadError = {
  subscribe: loadErrorStore.subscribe
};
