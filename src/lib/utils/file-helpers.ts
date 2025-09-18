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

export function getFileTypeFromExtension(filename: string): string {
  const ext = getFileExtension(filename);

  const typeMap: Record<string, string> = {
    // Structure files
    'pdb': 'structure',
    'psf': 'structure',
    'pqr': 'structure',

    // Parameter files
    'prm': 'parameters',
    'par': 'parameters',
    'top': 'parameters',

    // Configuration files
    'namd': 'configuration',
    'conf': 'configuration',
    'inp': 'configuration',

    // Trajectory files
    'dcd': 'trajectory',
    'xtc': 'trajectory',
    'trr': 'trajectory',

    // Log files
    'log': 'log',
    'out': 'log',

    // Analysis files
    'xvg': 'analysis',
    'dat': 'analysis',

    // Checkpoint files
    'coor': 'checkpoint',
    'vel': 'checkpoint',
    'xsc': 'checkpoint',
    'restart': 'checkpoint'
  };

  return typeMap[ext] || 'log';
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

export function parseMemoryString(memory: string): number {
  if (!memory) return 0;
  const cleanMemory = memory.toString().toLowerCase().replace(/\s+/g, '');
  const match = cleanMemory.match(/^(\d+(?:\.\d+)?)([a-z]*)/);

  if (!match) return 0;

  const value = parseFloat(match[1]);
  const unit = match[2] || 'gb';

  switch (unit) {
    case 'kb':
      return value / (1024 * 1024);
    case 'mb':
      return value / 1024;
    case 'gb':
    case '':
      return value;
    case 'tb':
      return value * 1024;
    default:
      return value; // Assume GB if unknown unit
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

export function validateWalltime(walltime: string): { isValid: boolean; hours: number; error?: string } {
  if (!walltime) {
    return { isValid: false, hours: 0, error: 'Walltime is required' };
  }

  const timeRegex = /^(\d{1,3}):([0-5]?\d):([0-5]?\d)$/;
  const match = walltime.match(timeRegex);

  if (!match) {
    return { isValid: false, hours: 0, error: 'Format must be HH:MM:SS (e.g., 24:00:00)' };
  }

  const hours = parseInt(match[1], 10);
  const minutes = parseInt(match[2], 10);
  const seconds = parseInt(match[3], 10);

  if (hours < 0 || minutes >= 60 || seconds >= 60) {
    return { isValid: false, hours: 0, error: 'Invalid time values' };
  }

  const totalHours = hours + minutes / 60 + seconds / 3600;

  if (totalHours <= 0) {
    return { isValid: false, hours: 0, error: 'Walltime must be greater than 0' };
  }

  return { isValid: true, hours: totalHours };
}