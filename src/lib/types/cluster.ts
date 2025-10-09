/**
 * Cluster configuration types matching Rust backend structures
 * These types mirror src-tauri/src/cluster.rs
 */

export type PartitionCategory = 'Compute' | 'GPU' | 'HighMemory' | 'Development' | 'Compile';
export type QosPriority = 'High' | 'Normal' | 'Low';

export interface PartitionSpec {
  id: string;
  name: string;
  title: string;
  description: string;
  nodes: string;
  cores_per_node: string;
  ram_per_core: string;
  max_walltime: string;
  gpu_type: string | null;
  gpu_count: number | null;
  category: PartitionCategory;
  use_cases: string[];
  is_standard: boolean;
  is_default: boolean;
}

export interface QosSpec {
  id: string;
  name: string;
  title: string;
  description: string;
  max_walltime_hours: number;
  max_jobs: number;
  node_limit: number;
  valid_partitions: string[];
  requirements: string[];
  priority: QosPriority;
  is_default: boolean;
}

export interface JobPresetConfig {
  cores: number;
  memory: string;
  wall_time: string;
  partition: string;
  qos: string;
}

export interface JobPreset {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  config: JobPresetConfig;
  estimated_cost: string;
  estimated_queue: string;
  use_cases: string[];
  requires_gpu: boolean;
}

export interface BillingRates {
  cpu_cost_per_core_hour: number;
  gpu_cost_per_gpu_hour: number;
}

export interface ClusterCapabilities {
  partitions: PartitionSpec[];
  qos_options: QosSpec[];
  job_presets: JobPreset[];
  billing_rates: BillingRates;
}

export interface PartitionLimits {
  max_cores: number;
  max_memory_per_core: number;
  min_memory_for_qos: number | null;
}

export interface ValidationResult {
  is_valid: boolean;
  issues: string[];
  warnings: string[];
  suggestions: string[];
}
