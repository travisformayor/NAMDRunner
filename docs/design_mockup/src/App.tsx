import { useState } from "react";
import { Sidebar } from "./components/layout/Sidebar";
import { ConnectionStatus } from "./components/layout/ConnectionStatus";
import { Breadcrumbs } from "./components/layout/Breadcrumbs";
import { SSHConsole } from "./components/layout/SSHConsole";
import { JobsView } from "./components/jobs/JobsView";
import { JobDetailView } from "./components/job-detail/JobDetailView";
import { CreateJobView } from "./components/job-create/CreateJobView";

export type View = "jobs" | "create" | "settings";
export type ConnectionState = "connected" | "connecting" | "disconnected" | "expired";

export interface Job {
  id: string;
  name: string;
  status: "CREATED" | "PENDING" | "RUNNING" | "COMPLETED" | "FAILED" | "CANCELLED";
  runtime: string;
  wallTime: string;
  wallTimeLeft: string;
  createdDate: string;
  submittedDate: string;
  jobId: string;
  slurmJobId?: string;
}

// Mock data for demonstration
const mockJobs: Job[] = [
  {
    id: "1",
    name: "protein_folding_simulation",
    status: "RUNNING",
    runtime: "02:15:30",
    wallTime: "4h total",
    wallTimeLeft: "45m left",
    createdDate: "2024-01-15 09:30",
    submittedDate: "2024-01-15 09:35",
    jobId: "job_001",
    slurmJobId: "12345678"
  },
  {
    id: "2", 
    name: "membrane_dynamics",
    status: "COMPLETED",
    runtime: "04:30:15",
    wallTime: "6h total",
    wallTimeLeft: "completed",
    createdDate: "2024-01-14 14:20",
    submittedDate: "2024-01-14 14:25",
    jobId: "job_002",
    slurmJobId: "12345677"
  },
  {
    id: "3",
    name: "drug_binding_analysis", 
    status: "PENDING",
    runtime: "--",
    wallTime: "8h total",
    wallTimeLeft: "pending",
    createdDate: "2024-01-15 11:45",
    submittedDate: "2024-01-15 11:50",
    jobId: "job_003",
    slurmJobId: "12345679"
  },
  {
    id: "4",
    name: "enzyme_kinetics",
    status: "FAILED", 
    runtime: "00:45:12",
    wallTime: "2h total",
    wallTimeLeft: "failed",
    createdDate: "2024-01-15 08:15",
    submittedDate: "2024-01-15 08:20",
    jobId: "job_004",
    slurmJobId: "12345676"
  }
];

export default function App() {
  const [currentView, setCurrentView] = useState<View>("jobs");
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>("connected");
  const [consoleOpen, setConsoleOpen] = useState(false);
  const [jobs] = useState<Job[]>(mockJobs);

  const selectedJob = selectedJobId ? jobs.find(job => job.id === selectedJobId) : null;

  const getBreadcrumbs = () => {
    if (currentView === "jobs" && selectedJob) {
      return [
        { label: "Jobs", onClick: () => { setCurrentView("jobs"); setSelectedJobId(null); } },
        { label: selectedJob.name }
      ];
    }
    if (currentView === "create") {
      return [
        { label: "Jobs", onClick: () => setCurrentView("jobs") },
        { label: "Create New Job" }
      ];
    }
    return [{ label: "Jobs" }];
  };

  const renderMainContent = () => {
    if (currentView === "jobs" && selectedJob) {
      return (
        <JobDetailView 
          job={selectedJob} 
          onBack={() => setSelectedJobId(null)}
        />
      );
    }
    
    if (currentView === "create") {
      return (
        <CreateJobView 
          onCancel={() => setCurrentView("jobs")}
          onJobCreated={() => setCurrentView("jobs")}
        />
      );
    }
    
    return (
      <JobsView 
        jobs={jobs}
        onJobSelect={(jobId) => setSelectedJobId(jobId)}
        connectionState={connectionState}
      />
    );
  };

  return (
    <div className="size-full flex bg-background">
      {/* Sidebar */}
      <Sidebar 
        currentView={currentView}
        onViewChange={(view) => {
          setCurrentView(view);
          setSelectedJobId(null);
        }}
      />
      
      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-h-0">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border bg-card">
          <Breadcrumbs items={getBreadcrumbs()} />
          <ConnectionStatus 
            state={connectionState}
            onStateChange={setConnectionState}
          />
        </div>
        
        {/* Content */}
        <div className="flex-1 overflow-auto">
          {renderMainContent()}
        </div>
        
        {/* SSH Console */}
        <SSHConsole 
          isOpen={consoleOpen}
          onToggle={() => setConsoleOpen(!consoleOpen)}
        />
      </div>
    </div>
  );
}