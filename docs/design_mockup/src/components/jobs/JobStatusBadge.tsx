import { Badge } from "../ui/badge";

type JobStatus = "CREATED" | "PENDING" | "RUNNING" | "COMPLETED" | "FAILED" | "CANCELLED";

interface JobStatusBadgeProps {
  status: JobStatus;
}

export function JobStatusBadge({ status }: JobStatusBadgeProps) {
  const getStatusConfig = (status: JobStatus) => {
    switch (status) {
      case "CREATED":
        return {
          label: "CREATED",
          className: "bg-gray-100 text-gray-800 hover:bg-gray-200"
        };
      case "PENDING":
        return {
          label: "PENDING", 
          className: "bg-yellow-100 text-yellow-800 hover:bg-yellow-200"
        };
      case "RUNNING":
        return {
          label: "RUNNING",
          className: "bg-blue-100 text-blue-800 hover:bg-blue-200"
        };
      case "COMPLETED":
        return {
          label: "COMPLETED",
          className: "bg-green-100 text-green-800 hover:bg-green-200"
        };
      case "FAILED":
        return {
          label: "FAILED",
          className: "bg-red-100 text-red-800 hover:bg-red-200"
        };
      case "CANCELLED":
        return {
          label: "CANCELLED",
          className: "bg-gray-100 text-gray-700 hover:bg-gray-200"
        };
      default:
        return {
          label: status,
          className: "bg-gray-100 text-gray-800 hover:bg-gray-200"
        };
    }
  };

  const config = getStatusConfig(status);

  return (
    <Badge 
      variant="secondary"
      className={`${config.className} font-medium px-2 py-1 text-xs rounded-full`}
    >
      {config.label}
    </Badge>
  );
}