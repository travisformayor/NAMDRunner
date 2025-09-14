import { Job } from "../../../App";
import { Button } from "../../ui/button";
import { Card, CardContent } from "../../ui/card";
import { Download, FileText, Database, BarChart3 } from "lucide-react";

interface OutputFilesTabProps {
  job: Job;
}

interface OutputFile {
  name: string;
  size: string;
  type: "trajectory" | "log" | "analysis" | "checkpoint";
  description: string;
  lastModified: string;
  available: boolean;
}

const getOutputFiles = (job: Job): OutputFile[] => {
  const baseFiles: OutputFile[] = [
    {
      name: "trajectory.dcd",
      size: "1.2 GB",
      type: "trajectory",
      description: "Molecular dynamics trajectory",
      lastModified: "2024-01-15 12:45",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "output.log",
      size: "45 MB",
      type: "log",
      description: "NAMD output log file",
      lastModified: "2024-01-15 12:45",
      available: job.status !== "CREATED" && job.status !== "PENDING"
    },
    {
      name: "energy.log",
      size: "23 MB",
      type: "analysis",
      description: "Energy analysis output",
      lastModified: "2024-01-15 12:45",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "restart.coor",
      size: "12 MB",
      type: "checkpoint",
      description: "Restart coordinates",
      lastModified: "2024-01-15 12:30",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "restart.vel",
      size: "12 MB",
      type: "checkpoint",
      description: "Restart velocities",
      lastModified: "2024-01-15 12:30",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    },
    {
      name: "restart.xsc",
      size: "1 KB",
      type: "checkpoint",
      description: "Extended system coordinates",
      lastModified: "2024-01-15 12:30",
      available: job.status === "COMPLETED" || job.status === "RUNNING"
    }
  ];

  if (job.status === "COMPLETED") {
    baseFiles.push({
      name: "analysis_summary.txt",
      size: "156 KB",
      type: "analysis",
      description: "Final analysis summary",
      lastModified: "2024-01-15 13:35",
      available: true
    });
  }

  return baseFiles;
};

export function OutputFilesTab({ job }: OutputFilesTabProps) {
  const outputFiles = getOutputFiles(job);

  const getFileIcon = (type: string) => {
    switch (type) {
      case "trajectory":
        return <Database className="w-4 h-4 text-blue-600" />;
      case "log":
        return <FileText className="w-4 h-4 text-green-600" />;
      case "analysis":
        return <BarChart3 className="w-4 h-4 text-purple-600" />;
      case "checkpoint":
        return <FileText className="w-4 h-4 text-orange-600" />;
      default:
        return <FileText className="w-4 h-4 text-gray-600" />;
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case "trajectory":
        return "Trajectory";
      case "log":
        return "Log";
      case "analysis":
        return "Analysis";
      case "checkpoint":
        return "Checkpoint";
      default:
        return "Unknown";
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case "trajectory":
        return "bg-blue-100 text-blue-800";
      case "log":
        return "bg-green-100 text-green-800";
      case "analysis":
        return "bg-purple-100 text-purple-800";
      case "checkpoint":
        return "bg-orange-100 text-orange-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  };

  const handleDownload = (fileName: string) => {
    console.log("Download file:", fileName);
  };

  const availableFiles = outputFiles.filter(file => file.available);
  const unavailableFiles = outputFiles.filter(file => !file.available);

  if (job.status === "CREATED" || job.status === "PENDING") {
    return (
      <div className="text-center py-8 text-muted-foreground">
        Output files will be available once the job starts running.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Available Files */}
      {availableFiles.length > 0 && (
        <div className="space-y-4">
          <h3 className="text-lg font-medium">Available Files</h3>
          
          <div className="grid gap-3">
            {availableFiles.map((file, index) => (
              <Card key={index} className="hover:bg-muted/50 transition-colors">
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-start gap-3 flex-1">
                      {getFileIcon(file.type)}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-sm font-medium">{file.name}</span>
                          <span className={`px-2 py-1 text-xs rounded-full ${getTypeColor(file.type)}`}>
                            {getTypeLabel(file.type)}
                          </span>
                        </div>
                        <div className="text-xs text-muted-foreground mb-1">
                          {file.description}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          Size: {file.size} • Modified: {file.lastModified}
                        </div>
                      </div>
                    </div>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDownload(file.name)}
                      className="flex items-center gap-2"
                    >
                      <Download className="w-3 h-3" />
                      Download
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* Unavailable Files */}
      {unavailableFiles.length > 0 && (
        <div className="space-y-4">
          <h3 className="text-lg font-medium">Expected Files</h3>
          <div className="text-sm text-muted-foreground mb-3">
            These files will be available once the simulation produces them.
          </div>
          
          <div className="grid gap-3">
            {unavailableFiles.map((file, index) => (
              <Card key={index} className="opacity-60">
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-start gap-3 flex-1">
                      {getFileIcon(file.type)}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-sm font-medium">{file.name}</span>
                          <span className={`px-2 py-1 text-xs rounded-full ${getTypeColor(file.type)}`}>
                            {getTypeLabel(file.type)}
                          </span>
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {file.description}
                        </div>
                      </div>
                    </div>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      disabled
                      className="flex items-center gap-2"
                    >
                      <Download className="w-3 h-3" />
                      Pending
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* File Summary */}
      <div className="mt-6 p-4 bg-muted/30 rounded-lg">
        <div className="text-sm">
          <div className="font-medium mb-2">Output Summary</div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs text-muted-foreground">
            <div>
              <span className="font-medium">Trajectory Files:</span> {outputFiles.filter(f => f.type === "trajectory").length}
            </div>
            <div>
              <span className="font-medium">Log Files:</span> {outputFiles.filter(f => f.type === "log").length}
            </div>
            <div>
              <span className="font-medium">Analysis Files:</span> {outputFiles.filter(f => f.type === "analysis").length}
            </div>
            <div>
              <span className="font-medium">Checkpoint Files:</span> {outputFiles.filter(f => f.type === "checkpoint").length}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}