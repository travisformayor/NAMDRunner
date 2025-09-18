/**
 * Alpine Cluster Configuration
 *
 * Centralized source of truth for all cluster-specific data.
 * Based on docs/cluster-guide.md - update this file when cluster config changes.
 *
 * Last Updated: 2025-01-15
 * Source: CU Research Computing (CURC) Alpine cluster documentation
 */

export interface PartitionSpec {
  id: string;
  name: string;
  title: string;
  description: string;
  nodes: string;
  coresPerNode: string;
  ramPerCore: string;
  maxWalltime: string;
  gpuType?: string;
  gpuCount?: number;
  category: 'compute' | 'gpu' | 'memory' | 'testing';
  useCases: string[];
  isStandard?: boolean;
}

export interface QosSpec {
  id: string;
  name: string;
  title: string;
  description: string;
  maxWalltimeHours: number;
  maxJobs: number;
  nodeLimit: number;
  validPartitions: string[];
  requirements?: string[];
  priority: 'Standard' | 'High';
}

export interface JobPreset {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: 'test' | 'production' | 'large' | 'gpu';
  config: {
    cores: number;
    memory: string;
    wallTime: string;
    partition: string;
    qos: string;
  };
  estimatedCost: string;
  estimatedQueue: string;
  useCases: string[];
  requiresGpu?: boolean;
}

// Hardware Partitions (Production)
export const PARTITIONS: PartitionSpec[] = [
  {
    id: 'amilan',
    name: 'amilan',
    title: 'General Compute (Default)',
    description: 'Standard CPU nodes for most NAMD simulations',
    nodes: '374+',
    coresPerNode: '32/48/64',
    ramPerCore: '3.75 GB',
    maxWalltime: '24H (7D with long QoS)',
    category: 'compute',
    useCases: ['Production runs', 'Standard simulations', 'Most NAMD jobs'],
    isStandard: true
  },
  {
    id: 'amilan128c',
    name: 'amilan128c',
    title: 'High-Core Compute',
    description: 'High core count nodes for large parallel jobs',
    nodes: '16+',
    coresPerNode: '128',
    ramPerCore: '2.01 GB',
    maxWalltime: '24H (7D with long QoS)',
    category: 'compute',
    useCases: ['Large simulations', 'Highly parallel jobs', '100+ core jobs']
  },
  {
    id: 'amem',
    name: 'amem',
    title: 'High-Memory',
    description: 'High memory nodes for memory-intensive simulations',
    nodes: '22+',
    coresPerNode: '48/64/128',
    ramPerCore: '16-21.5 GB',
    maxWalltime: '4H (7D with mem QoS)',
    category: 'memory',
    useCases: ['Large systems', 'Memory-intensive jobs', 'Complex simulations']
  },
  {
    id: 'aa100',
    name: 'aa100',
    title: 'NVIDIA A100 GPU',
    description: 'GPU-accelerated nodes with NVIDIA A100 for fast NAMD',
    nodes: '10+',
    coresPerNode: '64',
    ramPerCore: '3.75 GB',
    maxWalltime: '24H (7D with long QoS)',
    gpuType: 'NVIDIA A100',
    gpuCount: 3,
    category: 'gpu',
    useCases: ['GPU-accelerated NAMD', 'Fast simulations', 'CUDA workloads']
  },
  {
    id: 'ami100',
    name: 'ami100',
    title: 'AMD MI100 GPU',
    description: 'GPU-accelerated nodes with AMD MI100 for HIP workloads',
    nodes: '8+',
    coresPerNode: '64',
    ramPerCore: '3.75 GB',
    maxWalltime: '24H (7D with long QoS)',
    gpuType: 'AMD MI100',
    gpuCount: 3,
    category: 'gpu',
    useCases: ['HIP-accelerated workloads', 'AMD GPU computing', 'Alternative GPU option']
  },
  {
    id: 'al40',
    name: 'al40',
    title: 'NVIDIA L40 GPU',
    description: 'GPU-accelerated nodes with NVIDIA L40',
    nodes: '2+',
    coresPerNode: '64',
    ramPerCore: '3.75 GB',
    maxWalltime: '24H (7D with long QoS)',
    gpuType: 'NVIDIA L40',
    gpuCount: 3,
    category: 'gpu',
    useCases: ['GPU computing', 'Graphics workloads', 'AI/ML tasks']
  }
];

// Testing Partitions (Development Use)
export const TESTING_PARTITIONS: PartitionSpec[] = [
  {
    id: 'atesting',
    name: 'atesting',
    title: 'CPU Testing',
    description: 'Quick CPU testing with limited resources',
    nodes: '2',
    coresPerNode: '16 total',
    ramPerCore: 'Variable',
    maxWalltime: '1 hour',
    category: 'testing',
    useCases: ['Testing configurations', 'Quick validation', 'Development']
  },
  {
    id: 'atesting_a100',
    name: 'atesting_a100',
    title: 'GPU Testing (A100 MIG)',
    description: 'GPU testing with A100 MIG instances',
    nodes: '1',
    coresPerNode: '10',
    ramPerCore: 'Variable',
    maxWalltime: '1 hour',
    gpuType: 'NVIDIA A100 MIG',
    gpuCount: 1,
    category: 'testing',
    useCases: ['GPU testing', 'CUDA development', 'Quick GPU validation']
  },
  {
    id: 'atesting_mi100',
    name: 'atesting_mi100',
    title: 'GPU Testing (MI100)',
    description: 'GPU testing with AMD MI100',
    nodes: '1',
    coresPerNode: '64',
    ramPerCore: '3.75 GB',
    maxWalltime: '1 hour',
    gpuType: 'AMD MI100',
    gpuCount: 3,
    category: 'testing',
    useCases: ['HIP testing', 'AMD GPU development', 'GPU validation']
  },
  {
    id: 'acompile',
    name: 'acompile',
    title: 'Code Compilation',
    description: 'Dedicated nodes for compiling code',
    nodes: '1',
    coresPerNode: '4',
    ramPerCore: 'Variable',
    maxWalltime: '12 hours',
    category: 'testing',
    useCases: ['Code compilation', 'Build processes', 'Software development']
  }
];

// Quality of Service (QoS) Options
export const QOS_OPTIONS: QosSpec[] = [
  {
    id: 'normal',
    name: 'normal',
    title: 'Normal Priority (Default)',
    description: 'Standard priority with good resource limits',
    maxWalltimeHours: 24,
    maxJobs: 1000,
    nodeLimit: 128,
    validPartitions: ['amilan', 'amilan128c', 'aa100', 'ami100', 'al40'],
    priority: 'Standard'
  },
  {
    id: 'long',
    name: 'long',
    title: 'Extended Runtime',
    description: 'Longer walltime for extended simulations',
    maxWalltimeHours: 168, // 7 days
    maxJobs: 200,
    nodeLimit: 20,
    validPartitions: ['amilan', 'amilan128c', 'aa100', 'ami100', 'al40'],
    priority: 'Standard'
  },
  {
    id: 'mem',
    name: 'mem',
    title: 'High-Memory',
    description: 'For high-memory jobs on amem partition',
    maxWalltimeHours: 168, // 7 days
    maxJobs: 1000,
    nodeLimit: 12,
    validPartitions: ['amem'],
    requirements: ['256GB+ memory', 'amem partition only'],
    priority: 'Standard'
  },
  {
    id: 'testing',
    name: 'testing',
    title: 'Testing & Development',
    description: 'Quick testing with limited resources',
    maxWalltimeHours: 1,
    maxJobs: 5,
    nodeLimit: 2,
    validPartitions: ['atesting', 'atesting_a100', 'atesting_mi100'],
    priority: 'High'
  },
  {
    id: 'compile',
    name: 'compile',
    title: 'Compilation',
    description: 'For code compilation jobs',
    maxWalltimeHours: 12,
    maxJobs: 999, // Effectively unlimited
    nodeLimit: 1,
    validPartitions: ['acompile'],
    priority: 'Standard'
  }
];

// NAMD Job Presets (based on cluster-guide.md recommendations)
export const JOB_PRESETS: JobPreset[] = [
  {
    id: 'small-test',
    name: 'Small Test',
    description: 'Quick test for debugging and validation',
    icon: '🧪',
    category: 'test',
    config: {
      cores: 24,
      memory: '16',
      wallTime: '04:00:00',
      partition: 'amilan',
      qos: 'normal'
    },
    estimatedCost: '96 SU',
    estimatedQueue: '< 30 min',
    useCases: ['Testing configurations', 'Small systems', 'Quick validation']
  },
  {
    id: 'production',
    name: 'Production Run',
    description: 'Standard production simulation',
    icon: '⚡',
    category: 'production',
    config: {
      cores: 48,
      memory: '32',
      wallTime: '24:00:00',
      partition: 'amilan',
      qos: 'normal'
    },
    estimatedCost: '1,152 SU',
    estimatedQueue: '< 2 hours',
    useCases: ['Standard simulations', 'Production runs', 'Most NAMD jobs']
  },
  {
    id: 'large-scale',
    name: 'Large Scale',
    description: 'High-performance parallel simulation',
    icon: '🚀',
    category: 'large',
    config: {
      cores: 128,
      memory: '64',
      wallTime: '168:00:00', // 7 days
      partition: 'amilan128c',
      qos: 'long'
    },
    estimatedCost: '21,504 SU',
    estimatedQueue: '2-6 hours',
    useCases: ['Large systems', 'Long simulations', 'High-throughput jobs']
  },
  {
    id: 'gpu-accelerated',
    name: 'GPU Accelerated',
    description: 'Fast GPU-powered simulation',
    icon: '🔥',
    category: 'gpu',
    config: {
      cores: 64,
      memory: '48',
      wallTime: '24:00:00',
      partition: 'aa100',
      qos: 'normal'
    },
    estimatedCost: '4,107 SU', // 64 cores + 1 GPU estimate
    estimatedQueue: '1-4 hours',
    useCases: ['GPU-accelerated NAMD', 'Fast simulations', 'CUDA workloads'],
    requiresGpu: true
  }
];

// Resource Limits (for validation)
export const RESOURCE_LIMITS = {
  partition: {
    amilan: { maxCores: 64, maxMemoryPerCore: 3.75 },
    amilan128c: { maxCores: 128, maxMemoryPerCore: 2.01 },
    amem: { maxCores: 128, maxMemoryPerCore: 21.5, minMemoryForQos: 256 },
    aa100: { maxCores: 64, maxMemoryPerCore: 3.75 },
    ami100: { maxCores: 64, maxMemoryPerCore: 3.75 },
    al40: { maxCores: 64, maxMemoryPerCore: 3.75 },
    atesting: { maxCores: 16, maxMemoryPerCore: 4 },
    atesting_a100: { maxCores: 10, maxMemoryPerCore: 4 },
    atesting_mi100: { maxCores: 64, maxMemoryPerCore: 3.75 },
    acompile: { maxCores: 4, maxMemoryPerCore: 4 }
  }
};

// Billing Weights (for cost calculation)
export const BILLING_RATES = {
  cpuCostPerCoreHour: 1.0, // SU per core-hour
  gpuCostPerGpuHour: 108.2 // SU per GPU-hour
};

// Get all partitions (production + testing)
export function getAllPartitions(): PartitionSpec[] {
  return [...PARTITIONS, ...TESTING_PARTITIONS];
}

// Get partitions by category
export function getPartitionsByCategory(category: PartitionSpec['category']): PartitionSpec[] {
  return getAllPartitions().filter(p => p.category === category);
}

// Get QOS options for a partition
export function getQosForPartition(partitionId: string): QosSpec[] {
  return QOS_OPTIONS.filter(qos => qos.validPartitions.includes(partitionId));
}

// Get resource limits for a partition
export function getPartitionLimits(partitionId: string) {
  return RESOURCE_LIMITS.partition[partitionId as keyof typeof RESOURCE_LIMITS.partition];
}

// Calculate estimated cost
export function calculateJobCost(cores: number, walltimeHours: number, hasGpu: boolean = false, gpuCount: number = 1): number {
  const coreCost = cores * walltimeHours * BILLING_RATES.cpuCostPerCoreHour;
  const gpuCost = hasGpu ? gpuCount * walltimeHours * BILLING_RATES.gpuCostPerGpuHour : 0;
  return Math.round(coreCost + gpuCost);
}

// Suggest optimal QOS based on walltime and partition
export function suggestQos(walltimeHours: number, partitionId: string): string {
  const availableQos = getQosForPartition(partitionId);

  if (partitionId === 'amem') {
    return 'mem';
  } else if (partitionId.startsWith('atesting')) {
    return 'testing';
  } else if (partitionId === 'acompile') {
    return 'compile';
  } else if (walltimeHours > 24) {
    return availableQos.some(q => q.id === 'long') ? 'long' : 'normal';
  } else {
    return 'normal';
  }
}

// Convert walltime string to hours
export function walltimeToHours(walltime: string): number {
  if (!walltime) return 0;
  const [hours, minutes, seconds] = walltime.split(':').map(Number);
  return (hours || 0) + (minutes || 0) / 60 + (seconds || 0) / 3600;
}

// Enhanced resource validation using centralized configuration
export function validateResourceRequest(cores: number, memoryGB: number, walltimeHours: number, partitionId: string, qosId: string) {
  const issues: string[] = [];
  const warnings: string[] = [];
  const suggestions: string[] = [];

  // Get partition and QOS specs
  const partition = getAllPartitions().find(p => p.id === partitionId);
  const qos = QOS_OPTIONS.find(q => q.id === qosId);
  const limits = getPartitionLimits(partitionId);

  if (!partition) {
    issues.push(`Unknown partition: ${partitionId}`);
    return { isValid: false, issues, warnings, suggestions };
  }

  if (!qos) {
    issues.push(`Unknown QOS: ${qosId}`);
    return { isValid: false, issues, warnings, suggestions };
  }

  // Validate cores
  if (limits && cores > limits.maxCores) {
    issues.push(`Cores (${cores}) exceeds partition limit (${limits.maxCores})`);
  }

  // Validate memory
  if (limits && memoryGB > (cores * limits.maxMemoryPerCore)) {
    const maxMemory = cores * limits.maxMemoryPerCore;
    issues.push(`Memory (${memoryGB}GB) exceeds limit for ${cores} cores (${maxMemory.toFixed(1)}GB)`);
  }

  // Validate walltime
  if (walltimeHours > qos.maxWalltimeHours) {
    issues.push(`Walltime (${walltimeHours}h) exceeds QOS limit (${qos.maxWalltimeHours}h)`);
  }

  // QOS-specific validation
  if (qosId === 'mem' && memoryGB < 256) {
    issues.push('mem QOS requires at least 256GB memory');
  }

  if (!qos.validPartitions.includes(partitionId)) {
    issues.push(`QOS "${qosId}" is not valid for partition "${partitionId}"`);
  }

  // Efficiency warnings
  if (cores < 16) {
    warnings.push('Small core count may have longer queue times');
  }

  if (partitionId === 'amilan128c' && cores < 64) {
    warnings.push('Consider amilan partition for jobs under 64 cores');
  }

  if (walltimeHours > 48 && qosId === 'normal') {
    suggestions.push('Consider long QOS for runs over 48 hours');
  }

  // Memory optimization suggestions
  const recommendedMemory = cores * 2; // 2GB per core is often efficient
  if (memoryGB > recommendedMemory * 2) {
    suggestions.push(`Consider reducing memory to ~${recommendedMemory}GB for better efficiency`);
  }

  return {
    isValid: issues.length === 0,
    issues,
    warnings,
    suggestions
  };
}

// Estimate queue time based on resource request
export function estimateQueueTime(cores: number, partitionId: string): string {
  // GPU partitions generally have longer queues
  if (partitionId.includes('a100') || partitionId.includes('mi100') || partitionId.includes('l40')) {
    if (cores <= 32) return '1-4 hours';
    if (cores <= 64) return '4-8 hours';
    return '> 8 hours';
  }

  // Testing partitions are fast
  if (partitionId.startsWith('atesting') || partitionId === 'acompile') {
    return '< 15 minutes';
  }

  // Standard CPU partitions
  if (cores <= 24) return '< 30 minutes';
  if (cores <= 48) return '< 2 hours';
  if (cores <= 128) return '2-6 hours';
  return '> 6 hours';
}