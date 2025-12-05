// File type display utilities

interface FileTypeInfo {
  label: string;
  icon: string;
  color: string;
  description: string;
}

const FILE_TYPE_MAP: Record<string, FileTypeInfo> = {
  // Structure files
  pdb: { label: "Structure", icon: "🧬", color: "namd-file-type-structure", description: "Protein structure file" },
  psf: { label: "Structure", icon: "🧬", color: "namd-file-type-structure", description: "Protein structure file (PSF format)" },

  // Parameter files
  prm: { label: "Parameters", icon: "⚙️", color: "namd-file-type-parameters", description: "Parameter file" },
  rtf: { label: "Parameters", icon: "⚙️", color: "namd-file-type-parameters", description: "Parameter file" },
  str: { label: "Parameters", icon: "⚙️", color: "namd-file-type-parameters", description: "Parameter file" },

  // Configuration files
  conf: { label: "Configuration", icon: "📋", color: "namd-file-type-configuration", description: "Configuration file" },
  namd: { label: "Configuration", icon: "📋", color: "namd-file-type-configuration", description: "NAMD configuration file" },

  // Trajectory files
  dcd: { label: "Trajectory", icon: "📊", color: "namd-file-type-trajectory", description: "Trajectory data" },

  // Checkpoint files
  coor: { label: "Checkpoint", icon: "💾", color: "namd-file-type-checkpoint", description: "Coordinate checkpoint" },
  vel: { label: "Checkpoint", icon: "💾", color: "namd-file-type-checkpoint", description: "Velocity checkpoint" },
  xsc: { label: "Checkpoint", icon: "💾", color: "namd-file-type-checkpoint", description: "Extended system checkpoint" },

  // Log files
  log: { label: "Log", icon: "📄", color: "namd-file-type-log", description: "Log file" },

  // Other
  other: { label: "Other", icon: "📄", color: "namd-file-type-default", description: "Data file" }
};

const DEFAULT_FILE_TYPE: FileTypeInfo = {
  label: "Unknown",
  icon: "📄",
  color: "namd-file-type-default",
  description: "Data file"
};

function getFileTypeInfo(extensionOrType: string): FileTypeInfo {
  return FILE_TYPE_MAP[extensionOrType.toLowerCase()] || DEFAULT_FILE_TYPE;
}

export function getFileIcon(extensionOrType: string): string {
  return getFileTypeInfo(extensionOrType).icon;
}

export function getFileExtension(filename: string): string {
  return filename.split('.').pop()?.toLowerCase() || 'other';
}

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
