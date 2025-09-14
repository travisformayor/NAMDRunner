import { Job } from "../../../App";
import { Button } from "../../ui/button";
import { Card, CardContent } from "../../ui/card";
import { Download, FileText, Settings } from "lucide-react";

interface InputFilesTabProps {
  job: Job;
}

interface FileInfo {
  name: string;
  size: string;
  type: "structure" | "parameters" | "configuration";
  description: string;
}

const mockInputFiles: FileInfo[] = [
  {
    name: "protein.pdb",
    size: "2.3 MB",
    type: "structure",
    description: "Protein structure file"
  },
  {
    name: "protein.psf",
    size: "1.8 MB", 
    type: "structure",
    description: "Protein structure file (PSF format)"
  },
  {
    name: "par_all36_prot.prm",
    size: "456 KB",
    type: "parameters",
    description: "CHARMM36 protein parameters"
  },
  {
    name: "par_all36_lipid.prm",
    size: "234 KB",
    type: "parameters", 
    description: "CHARMM36 lipid parameters"
  },
  {
    name: "toppar_water_ions.str",
    size: "123 KB",
    type: "parameters",
    description: "Water and ion parameters"
  },
  {
    name: "simulation.conf",
    size: "4.2 KB",
    type: "configuration",
    description: "NAMD configuration file"
  }
];

export function InputFilesTab({ job }: InputFilesTabProps) {
  const getFileIcon = (type: string) => {
    switch (type) {
      case "structure":
        return <FileText className="w-4 h-4 text-blue-600" />;
      case "parameters":
        return <Settings className="w-4 h-4 text-green-600" />;
      case "configuration":
        return <Settings className="w-4 h-4 text-purple-600" />;
      default:
        return <FileText className="w-4 h-4 text-gray-600" />;
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case "structure":
        return "Structure";
      case "parameters":
        return "Parameters";
      case "configuration":
        return "Configuration";
      default:
        return "Unknown";
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case "structure":
        return "bg-blue-100 text-blue-800";
      case "parameters":
        return "bg-green-100 text-green-800";
      case "configuration":
        return "bg-purple-100 text-purple-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  };

  const handleDownload = (fileName: string) => {
    console.log("Download file:", fileName);
  };

  return (
    <div className="space-y-4">
      <div className="text-sm text-muted-foreground">
        Input files used for job: {job.name}
      </div>

      <div className="grid gap-3">
        {mockInputFiles.map((file, index) => (
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
                      Size: {file.size}
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

      <div className="mt-6 p-4 bg-muted/30 rounded-lg">
        <div className="text-sm">
          <div className="font-medium mb-2">File Summary</div>
          <div className="grid grid-cols-3 gap-4 text-xs text-muted-foreground">
            <div>
              <span className="font-medium">Structure Files:</span> 2
            </div>
            <div>
              <span className="font-medium">Parameter Files:</span> 3
            </div>
            <div>
              <span className="font-medium">Configuration Files:</span> 1
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}