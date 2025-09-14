import { useState } from "react";
import { Job } from "../../App";
import { JobSummaryCard } from "./JobSummaryCard";
import { JobTabs } from "./JobTabs";
import { Button } from "../ui/button";
import { ArrowLeft, Download, Trash2 } from "lucide-react";
import { DeleteJobDialog } from "./DeleteJobDialog";

interface JobDetailViewProps {
  job: Job;
  onBack: () => void;
}

export function JobDetailView({ job, onBack }: JobDetailViewProps) {
  const [activeTab, setActiveTab] = useState("overview");
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const handleSyncResults = () => {
    console.log("Sync results from scratch");
  };

  const handleDeleteJob = (deleteFiles: boolean) => {
    console.log("Delete job", job.id, "with files:", deleteFiles);
    setShowDeleteDialog(false);
    onBack();
  };

  return (
    <div className="p-6 space-y-6">
      {/* Back Button */}
      <Button
        variant="ghost"
        onClick={onBack}
        className="mb-4"
      >
        <ArrowLeft className="w-4 h-4 mr-2" />
        Back to Jobs
      </Button>

      {/* Job Summary */}
      <JobSummaryCard job={job} />

      {/* Tab Navigation */}
      <JobTabs 
        activeTab={activeTab}
        onTabChange={setActiveTab}
        job={job}
      />

      {/* Action Buttons */}
      <div className="flex items-center gap-3 pt-4 border-t border-border">
        {job.status === "COMPLETED" && (
          <Button
            onClick={handleSyncResults}
            className="flex items-center gap-2"
          >
            <Download className="w-4 h-4" />
            Sync Results from Scratch
          </Button>
        )}
        
        <Button
          variant="destructive"
          onClick={() => setShowDeleteDialog(true)}
          className="flex items-center gap-2"
        >
          <Trash2 className="w-4 h-4" />
          Delete Job
        </Button>
      </div>

      {/* Delete Confirmation Dialog */}
      <DeleteJobDialog
        isOpen={showDeleteDialog}
        jobName={job.name}
        onClose={() => setShowDeleteDialog(false)}
        onConfirm={handleDeleteJob}
      />
    </div>
  );
}