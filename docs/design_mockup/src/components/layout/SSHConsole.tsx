import { useState } from "react";
import { Button } from "../ui/button";
import { ChevronUp, ChevronDown, Copy, X } from "lucide-react";
import { ScrollArea } from "../ui/scroll-area";

interface SSHConsoleProps {
  isOpen: boolean;
  onToggle: () => void;
}

const mockConsoleOutput = [
  "$ module load slurm/alpine",
  '$ squeue -u jsmith --format="%.10i %.20j %.8T %.10M %.6D %R"',
  "    JOBID                 NAME     STATE       TIME  NODES NODELIST(REASON)",
  "12345678     protein_folding_sim        R    2:15:30      4 compute-[001-004]",
  "12345679        drug_binding_ana       PD       0:00      2 (Resources)",
  "$ sbatch job.sbatch",
  "Submitted batch job 12345680",
  "$ scancel 12345676",
  "Job cancelled successfully",
  "$ scontrol show job 12345678",
  "JobId=12345678 JobName=protein_folding_sim",
  "   UserId=jsmith(1001) GroupId=research(1001) MCS_label=N/A",
  "   Priority=1000 Nice=0 Account=research QOS=normal",
  "   JobState=RUNNING Reason=None Dependency=(null)",
  "   Requeue=1 Restarts=0 BatchFlag=1 Reboot=0 ExitCode=0:0",
  "   RunTime=02:15:30 TimeLimit=04:00:00 TimeMin=N/A",
  "   SubmitTime=2024-01-15T09:35:00 EligibleTime=2024-01-15T09:35:00",
  "   AccrueTime=2024-01-15T09:35:00",
  "   StartTime=2024-01-15T09:35:00 EndTime=2024-01-15T13:35:00 Deadline=N/A",
  "   SuspendTime=None SecsPreSuspend=0 LastSchedEval=2024-01-15T09:35:00",
  "   Partition=amilan AllocNode:Sid=login01:12345",
  "   ReqNodeList=(null) ExcNodeList=(null)",
  "   NodeList=compute-[001-004]",
  "   BatchHost=compute-001",
  "   NumNodes=4 NumCPUs=128 NumTasks=128 CPUs/Task=1 ReqB:S:C:T=0:0:*:*",
  "$",
];

export function SSHConsole({
  isOpen,
  onToggle,
}: SSHConsoleProps) {
  const [output] = useState(mockConsoleOutput);

  const handleCopyAll = () => {
    navigator.clipboard.writeText(output.join("\n"));
  };

  const handleClear = () => {
    // In a real implementation, this would clear the console output
    console.log("Clear console");
  };

  if (!isOpen) {
    return (
      <div className="border-t border-border bg-card">
        <Button
          variant="ghost"
          size="sm"
          onClick={onToggle}
          className="w-full justify-start p-2 rounded-none"
        >
          <ChevronUp className="w-4 h-4 mr-2" />
          SSH Console
        </Button>
      </div>
    );
  }

  return (
    <div
      className="border-t border-border bg-card"
      style={{ height: "33vh" }}
    >
      <div className="flex items-center justify-between p-2 border-b border-border">
        <Button
          variant="ghost"
          size="sm"
          onClick={onToggle}
          className="flex items-center gap-2 p-0 h-auto hover:bg-transparent"
        >
          <ChevronDown className="w-4 h-4" />
          <span className="text-sm font-medium">
            SSH Console
          </span>
        </Button>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleCopyAll}
            className="h-8 px-2"
          >
            <Copy className="w-4 h-4 mr-1" />
            Copy All
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClear}
            className="h-8 px-2"
          >
            <X className="w-4 h-4 mr-1" />
            Clear
          </Button>
        </div>
      </div>

      <ScrollArea className="h-full p-3">
        <div className="font-mono text-sm space-y-1">
          {output.map((line, index) => (
            <div key={index} className="whitespace-pre-wrap">
              {line}
            </div>
          ))}
          <div className="inline-block w-2 h-4 bg-foreground animate-pulse ml-1" />
        </div>
      </ScrollArea>
    </div>
  );
}