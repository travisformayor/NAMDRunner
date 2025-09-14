import { useState } from "react";
import { Job } from "../../App";
import { JobStatusBadge } from "./JobStatusBadge";
import { ChevronUp, ChevronDown } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";

interface JobTableProps {
  jobs: Job[];
  onJobSelect: (jobId: string) => void;
}

type SortField = "name" | "status" | "runtime" | "wallTime" | "createdDate" | "submittedDate" | "jobId";
type SortDirection = "asc" | "desc";

export function JobTable({ jobs, onJobSelect }: JobTableProps) {
  const [sortField, setSortField] = useState<SortField>("createdDate");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDirection("asc");
    }
  };

  const sortedJobs = [...jobs].sort((a, b) => {
    let aValue: string | number;
    let bValue: string | number;

    switch (sortField) {
      case "name":
        aValue = a.name;
        bValue = b.name;
        break;
      case "status":
        aValue = a.status;
        bValue = b.status;
        break;
      case "runtime":
        aValue = a.runtime === "--" ? "" : a.runtime;
        bValue = b.runtime === "--" ? "" : b.runtime;
        break;
      case "wallTime":
        aValue = a.wallTime;
        bValue = b.wallTime;
        break;
      case "createdDate":
        aValue = new Date(a.createdDate).getTime();
        bValue = new Date(b.createdDate).getTime();
        break;
      case "submittedDate":
        aValue = new Date(a.submittedDate).getTime();
        bValue = new Date(b.submittedDate).getTime();
        break;
      case "jobId":
        aValue = a.jobId;
        bValue = b.jobId;
        break;
      default:
        aValue = a.createdDate;
        bValue = b.createdDate;
    }

    if (typeof aValue === "string" && typeof bValue === "string") {
      const comparison = aValue.localeCompare(bValue);
      return sortDirection === "asc" ? comparison : -comparison;
    }

    if (typeof aValue === "number" && typeof bValue === "number") {
      return sortDirection === "asc" ? aValue - bValue : bValue - aValue;
    }

    return 0;
  });

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null;
    return sortDirection === "asc" ? (
      <ChevronUp className="w-4 h-4 inline ml-1" />
    ) : (
      <ChevronDown className="w-4 h-4 inline ml-1" />
    );
  };

  return (
    <div className="border border-border rounded-lg">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("name")}
            >
              Job Name <SortIcon field="name" />
            </TableHead>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("status")}
            >
              Status <SortIcon field="status" />
            </TableHead>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("runtime")}
            >
              Runtime <SortIcon field="runtime" />
            </TableHead>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("wallTime")}
            >
              Wall Time <SortIcon field="wallTime" />
            </TableHead>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("createdDate")}
            >
              Created Date <SortIcon field="createdDate" />
            </TableHead>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("submittedDate")}
            >
              Submitted Date <SortIcon field="submittedDate" />
            </TableHead>
            <TableHead 
              className="cursor-pointer select-none hover:bg-muted/50"
              onClick={() => handleSort("jobId")}
            >
              Job ID <SortIcon field="jobId" />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sortedJobs.map((job) => (
            <TableRow 
              key={job.id}
              className="cursor-pointer hover:bg-muted/50"
              onClick={() => onJobSelect(job.id)}
            >
              <TableCell className="font-medium">{job.name}</TableCell>
              <TableCell>
                <JobStatusBadge status={job.status} />
              </TableCell>
              <TableCell className="font-mono text-sm">{job.runtime}</TableCell>
              <TableCell>
                <div>
                  <div>{job.wallTime}</div>
                  {job.wallTimeLeft && job.wallTimeLeft !== "completed" && job.wallTimeLeft !== "failed" && job.wallTimeLeft !== "pending" && (
                    <div className="text-xs text-muted-foreground">{job.wallTimeLeft}</div>
                  )}
                </div>
              </TableCell>
              <TableCell>{job.createdDate}</TableCell>
              <TableCell>{job.submittedDate}</TableCell>
              <TableCell className="font-mono text-sm">{job.jobId}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}