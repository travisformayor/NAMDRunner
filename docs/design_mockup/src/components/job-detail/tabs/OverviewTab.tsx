import { Job } from "../../../App";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";
import { Progress } from "../../ui/progress";

interface OverviewTabProps {
  job: Job;
}

export function OverviewTab({ job }: OverviewTabProps) {
  // Mock data for demonstration
  const resourceUsage = {
    cpuUsage: 85,
    memoryUsage: 67,
    gpuUsage: job.status === "RUNNING" ? 92 : 0,
    diskUsage: 45
  };

  const statistics = {
    totalSteps: 1000000,
    completedSteps: job.status === "RUNNING" ? 450000 : (job.status === "COMPLETED" ? 1000000 : 0),
    avgStepTime: "0.025",
    estimatedCompletion: job.status === "RUNNING" ? "2024-01-15 14:20" : null
  };

  const getProgressPercentage = () => {
    if (statistics.totalSteps === 0) return 0;
    return (statistics.completedSteps / statistics.totalSteps) * 100;
  };

  return (
    <div className="space-y-6">
      {/* Simulation Progress */}
      {job.status === "RUNNING" || job.status === "COMPLETED" ? (
        <Card>
          <CardHeader>
            <CardTitle>Simulation Progress</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span>Steps: {statistics.completedSteps.toLocaleString()} / {statistics.totalSteps.toLocaleString()}</span>
                <span>{getProgressPercentage().toFixed(1)}%</span>
              </div>
              <Progress value={getProgressPercentage()} className="h-2" />
            </div>
            
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <div className="text-muted-foreground">Average Step Time</div>
                <div className="font-mono">{statistics.avgStepTime} ms</div>
              </div>
              {statistics.estimatedCompletion && (
                <div>
                  <div className="text-muted-foreground">Est. Completion</div>
                  <div>{statistics.estimatedCompletion}</div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      ) : null}

      {/* Resource Usage */}
      {job.status === "RUNNING" || job.status === "COMPLETED" ? (
        <Card>
          <CardHeader>
            <CardTitle>Resource Usage</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-6">
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span>CPU Usage</span>
                  <span>{resourceUsage.cpuUsage}%</span>
                </div>
                <Progress value={resourceUsage.cpuUsage} className="h-2" />
              </div>
              
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span>Memory Usage</span>
                  <span>{resourceUsage.memoryUsage}%</span>
                </div>
                <Progress value={resourceUsage.memoryUsage} className="h-2" />
              </div>
              
              {resourceUsage.gpuUsage > 0 && (
                <div>
                  <div className="flex justify-between text-sm mb-2">
                    <span>GPU Usage</span>
                    <span>{resourceUsage.gpuUsage}%</span>
                  </div>
                  <Progress value={resourceUsage.gpuUsage} className="h-2" />
                </div>
              )}
              
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span>Disk Usage</span>
                  <span>{resourceUsage.diskUsage}%</span>
                </div>
                <Progress value={resourceUsage.diskUsage} className="h-2" />
              </div>
            </div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="py-8 text-center text-muted-foreground">
            Resource usage information will be available once the job starts running.
          </CardContent>
        </Card>
      )}

      {/* Job Information */}
      <Card>
        <CardHeader>
          <CardTitle>Job Information</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-6 text-sm">
            <div>
              <div className="text-muted-foreground">Job Name</div>
              <div>{job.name}</div>
            </div>
            
            <div>
              <div className="text-muted-foreground">Status</div>
              <div>{job.status}</div>
            </div>
            
            <div>
              <div className="text-muted-foreground">Job ID</div>
              <div className="font-mono">{job.jobId}</div>
            </div>
            
            {job.slurmJobId && (
              <div>
                <div className="text-muted-foreground">SLURM Job ID</div>
                <div className="font-mono">{job.slurmJobId}</div>
              </div>
            )}
            
            <div>
              <div className="text-muted-foreground">Runtime</div>
              <div className="font-mono">{job.runtime}</div>
            </div>
            
            <div>
              <div className="text-muted-foreground">Wall Time</div>
              <div>{job.wallTime}</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}