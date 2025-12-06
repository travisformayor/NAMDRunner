// File and status display utilities

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
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
