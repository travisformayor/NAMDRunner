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
  FileUpload,
  UploadResult,
  DownloadResult,
  ListFilesResult,
  JobId,
} from '../types/api';

/**
 * Production implementation of the core client using Tauri IPC
 */
export class TauriCoreClient implements ICoreClient {
  // Connection management
  async connect(params: ConnectParams): Promise<ConnectResult> {
    return invoke('connect_to_cluster', { params });
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

  async submitJob(jobId: JobId): Promise<SubmitJobResult> {
    return invoke('submit_job', { jobId });
  }

  async getJobStatus(jobId: JobId): Promise<JobStatusResult> {
    return invoke('get_job_status', { jobId });
  }

  async getAllJobs(): Promise<GetAllJobsResult> {
    return invoke('get_all_jobs');
  }

  async syncJobs(): Promise<SyncJobsResult> {
    return invoke('sync_jobs');
  }

  async deleteJob(jobId: JobId, deleteRemote: boolean): Promise<DeleteJobResult> {
    return invoke('delete_job', { jobId, deleteRemote });
  }

  // File management
  async uploadJobFiles(jobId: JobId, files: FileUpload[]): Promise<UploadResult> {
    return invoke('upload_job_files', { jobId, files });
  }

  async downloadJobOutput(jobId: JobId, fileName: string): Promise<DownloadResult> {
    return invoke('download_job_output', { jobId, fileName });
  }

  async listJobFiles(jobId: JobId): Promise<ListFilesResult> {
    return invoke('list_job_files', { jobId });
  }
}