import { Job } from "../../App";
import { JobStatusBadge } from "../jobs/JobStatusBadge";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";

interface JobSummaryCardProps {
  job: Job;
}

export function JobSummaryCard({ job }: JobSummaryCardProps) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-xl">{job.name}</CardTitle>
            <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
              <span>Job ID: <span className="font-mono">{job.jobId}</span></span>
              {job.slurmJobId && (
                <span>SLURM ID: <span className="font-mono">{job.slurmJobId}</span></span>
              )}
            </div>
          </div>
          <JobStatusBadge status={job.status} />
        </div>
      </CardHeader>
      
      <CardContent>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div>
            <div className="text-sm text-muted-foreground">Created</div>
            <div className="text-sm">{job.createdDate}</div>
          </div>
          
          <div>
            <div className="text-sm text-muted-foreground">Submitted</div>
            <div className="text-sm">{job.submittedDate}</div>
          </div>
          
          <div>
            <div className="text-sm text-muted-foreground">Runtime</div>
            <div className="text-sm font-mono">{job.runtime}</div>
          </div>
          
          <div>
            <div className="text-sm text-muted-foreground">Wall Time</div>
            <div className="text-sm">
              <div>{job.wallTime}</div>
              {job.wallTimeLeft && job.wallTimeLeft !== "completed" && job.wallTimeLeft !== "failed" && job.wallTimeLeft !== "pending" && (
                <div className="text-xs text-muted-foreground">{job.wallTimeLeft}</div>
              )}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}