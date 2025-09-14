import { useState } from "react";
import { Job, ConnectionState } from "../../App";
import { Button } from "../ui/button";
import { JobTable } from "./JobTable";
import { SyncStatus } from "./SyncStatus";
import { Plus } from "lucide-react";

interface JobsViewProps {
  jobs: Job[];
  onJobSelect: (jobId: string) => void;
  connectionState: ConnectionState;
}

export function JobsView({ jobs, onJobSelect, connectionState }: JobsViewProps) {
  const [autoSync, setAutoSync] = useState(false);
  const [syncInterval, setSyncInterval] = useState(5);
  const [lastSyncTime] = useState(new Date(Date.now() - 5 * 60 * 1000)); // 5 minutes ago

  const handleCreateJob = () => {
    // This will be handled by the parent component
    console.log("Create job clicked");
  };

  const handleSyncNow = () => {
    console.log("Sync now clicked");
  };

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1>Jobs</h1>
        <Button 
          onClick={handleCreateJob}
          disabled={connectionState !== "connected"}
          className="flex items-center gap-2"
        >
          <Plus className="w-4 h-4" />
          Create New Job
        </Button>
      </div>

      {/* Sync Status */}
      <SyncStatus
        lastSyncTime={lastSyncTime}
        autoSync={autoSync}
        syncInterval={syncInterval}
        connectionState={connectionState}
        onSyncNow={handleSyncNow}
        onAutoSyncChange={setAutoSync}
        onSyncIntervalChange={setSyncInterval}
      />

      {/* Jobs Table */}
      <JobTable 
        jobs={jobs}
        onJobSelect={onJobSelect}
      />
    </div>
  );
}