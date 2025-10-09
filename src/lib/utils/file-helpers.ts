// File type display utilities

export function getFileIcon(type: string): string {
  const icons: Record<string, string> = {
    structure: "🧬",
    parameters: "⚙️",
    configuration: "📋",
    trajectory: "📊",
    log: "📄",
    analysis: "📈",
    checkpoint: "💾"
  };
  return icons[type] || "📄";
}

export function getTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    structure: "Structure",
    parameters: "Parameters",
    configuration: "Configuration",
    trajectory: "Trajectory",
    log: "Log",
    analysis: "Analysis",
    checkpoint: "Checkpoint"
  };
  return labels[type] || "Unknown";
}

export function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    structure: "namd-file-type-structure",
    parameters: "namd-file-type-parameters",
    configuration: "namd-file-type-configuration",
    trajectory: "namd-file-type-trajectory",
    log: "namd-file-type-log",
    analysis: "namd-file-type-analysis",
    checkpoint: "namd-file-type-checkpoint"
  };
  return colors[type] || "namd-file-type-default";
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function getFileExtension(filename: string): string {
  return filename.split('.').pop()?.toLowerCase() || '';
}

export function getStatusBadgeClass(status: string): string {
  return `namd-status-badge--${status.toLowerCase()}`;
}

export function getStatusInfo(status: string) {
  switch (status.toUpperCase()) {
    case 'CREATED':
      return {
        label: 'Created',
        class: 'created',
        icon: '📝'
      };
    case 'PENDING':
      return {
        label: 'Pending',
        class: 'pending',
        icon: '⏳'
      };
    case 'RUNNING':
      return {
        label: 'Running',
        class: 'running',
        icon: '▶️'
      };
    case 'COMPLETED':
      return {
        label: 'Completed',
        class: 'completed',
        icon: '✅'
      };
    case 'FAILED':
      return {
        label: 'Failed',
        class: 'failed',
        icon: '❌'
      };
    case 'CANCELLED':
      return {
        label: 'Cancelled',
        class: 'cancelled',
        icon: '🚫'
      };
    default:
      return {
        label: 'Unknown',
        class: 'unknown',
        icon: '❓'
      };
  }
}

export function formatMemory(gb: number): string {
  if (gb >= 1024) {
    return `${(gb / 1024).toFixed(1)}TB`;
  } else if (gb >= 1) {
    return `${gb.toFixed(gb % 1 === 0 ? 0 : 1)}GB`;
  } else {
    return `${(gb * 1024).toFixed(0)}MB`;
  }
}