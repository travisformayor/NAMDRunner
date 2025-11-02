import { invoke } from '@tauri-apps/api/core';
import type { ICoreClient } from './coreClient';
import type {
  ConnectParams,
  ConnectResult,
  DisconnectResult,
  ConnectionStatusResult,
  CreateJobParams,
  CreateJobResult,
  SubmitJobResult,
  JobStatusResult,
  GetAllJobsResult,
  SyncJobsResult,
  DeleteJobResult,
  RefetchLogsResult,
  FileUpload,
  UploadResult,
  DownloadResult,
  ListFilesResult,
  GetClusterCapabilitiesResult,
  ValidateResourceAllocationResult,
  JobId,
} from '../types/api';

/**
 * Production implementation of the core client using Tauri IPC
 */
export class TauriCoreClient implements ICoreClient {
  // Connection management
  async connect(params: ConnectParams): Promise<ConnectResult> {
    if (typeof window !== 'undefined' && window.sshConsole) {
      window.sshConsole.addDebug(`[SSH] Starting connection attempt to ${params.host} as ${params.username}`);
    }
    try {
      // Pass parameters as nested object under 'params' key
      const result = await invoke<ConnectResult>('connect_to_cluster', {
        params: {
          host: params.host,
          username: params.username,
          password: params.password
        }
      });
      if (typeof window !== 'undefined' && window.sshConsole) {
        window.sshConsole.addDebug(`[SSH] Connection result: ${result.success ? 'SUCCESS' : 'FAILED'}`);
        if (!result.success && result.error) {
          window.sshConsole.addDebug(`[SSH] Connection error: ${result.error}`);
        }
      }
      return result;
    } catch (error) {
      if (typeof window !== 'undefined' && window.sshConsole) {
        window.sshConsole.addDebug(`[SSH] Connection threw exception: ${error}`);
      }
      // Convert exception to ConnectResult instead of throwing
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Connection failed due to unexpected error'
      };
    }
  }

  async disconnect(): Promise<DisconnectResult> {
    return invoke('disconnect');
  }

  async getConnectionStatus(): Promise<ConnectionStatusResult> {
    return invoke('get_connection_status');
  }

  // Job management
  async createJob(params: CreateJobParams): Promise<CreateJobResult> {
    return invoke('create_job', { params });
  }

  async submitJob(job_id: JobId): Promise<SubmitJobResult> {
    return invoke('submit_job', { job_id });
  }

  async getJobStatus(job_id: JobId): Promise<JobStatusResult> {
    return invoke('get_job_status', { job_id });
  }

  async getAllJobs(): Promise<GetAllJobsResult> {
    return invoke('get_all_jobs');
  }

  async syncJobs(): Promise<SyncJobsResult> {
    return invoke('sync_jobs');
  }

  async deleteJob(job_id: JobId, delete_remote: boolean): Promise<DeleteJobResult> {
    return invoke('delete_job', { job_id, delete_remote });
  }

  async refetchSlurmLogs(job_id: JobId): Promise<RefetchLogsResult> {
    return invoke('refetch_slurm_logs', { job_id });
  }

  // File management
  async selectInputFiles(): Promise<any[]> {
    return invoke('select_input_files');
  }

  async detectFileType(filename: string): Promise<string> {
    return invoke('detect_file_type', { filename });
  }

  async uploadJobFiles(job_id: JobId, files: FileUpload[]): Promise<UploadResult> {
    return invoke('upload_job_files', { job_id, files });
  }

  async downloadJobOutput(job_id: JobId, file_path: string): Promise<DownloadResult> {
    return invoke('download_job_output', { job_id, file_path });
  }

  async downloadAllOutputs(job_id: JobId): Promise<DownloadResult> {
    return invoke('download_all_outputs', { job_id });
  }

  async listJobFiles(job_id: JobId): Promise<ListFilesResult> {
    return invoke('list_job_files', { job_id });
  }

  async getClusterCapabilities(): Promise<GetClusterCapabilitiesResult> {
    return invoke('get_cluster_capabilities');
  }

  async validateResourceAllocation(cores: number, memory: string, walltime: string, partition_id: string, qos_id: string): Promise<ValidateResourceAllocationResult> {
    return invoke('validate_resource_allocation', {
      cores,
      memory,
      walltime,
      partition_id,
      qos_id
    });
  }

  async suggestQosForPartition(walltime_hours: number, partition_id: string): Promise<string> {
    return invoke('suggest_qos_for_partition', {
      walltime_hours,
      partition_id
    });
  }

  async estimateQueueTimeForJob(cores: number, partition_id: string): Promise<string> {
    return invoke('estimate_queue_time_for_job', {
      cores,
      partition_id
    });
  }

  async calculateJobCost(cores: number, walltime_hours: number, has_gpu: boolean, gpu_count: number): Promise<number> {
    return invoke('calculate_job_cost', {
      cores,
      walltime_hours,
      has_gpu,
      gpu_count
    });
  }
}