import { useState } from "react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { ResourceForm } from "./ResourceForm";
import { FileUploadArea } from "./FileUploadArea";
import { NAMDConfigForm } from "./NAMDConfigForm";
import { ArrowLeft } from "lucide-react";

interface CreateJobViewProps {
  onCancel: () => void;
  onJobCreated: () => void;
}

export interface ResourceConfig {
  cores: number;
  memory: string;
  wallTime: string;
  partition: string;
  qos: string;
}

export interface UploadedFile {
  name: string;
  size: number;
  type: string;
  file: File;
}

export interface NAMDConfig {
  jobName: string;
  simulationSteps: number;
  temperature: number;
  timestep: number;
  outputName: string;
  dcdFreq?: number;
  restartFreq?: number;
}

export function CreateJobView({ onCancel, onJobCreated }: CreateJobViewProps) {
  const [resourceConfig, setResourceConfig] = useState<ResourceConfig>({
    cores: 128,
    memory: "512",
    wallTime: "04:00:00",
    partition: "amilan",
    qos: "normal"
  });

  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([]);
  
  const [namdConfig, setNAMDConfig] = useState<NAMDConfig>({
    jobName: "",
    simulationSteps: 1000000,
    temperature: 310,
    timestep: 2,
    outputName: "",
    dcdFreq: 5000,
    restartFreq: 10000
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    // Resource validation
    if (!resourceConfig.cores || resourceConfig.cores <= 0) {
      newErrors.cores = "Cores must be a positive number";
    }
    if (!resourceConfig.memory || parseFloat(resourceConfig.memory) <= 0) {
      newErrors.memory = "Memory must be a positive number";
    }
    if (!resourceConfig.wallTime || !/^\d{2}:\d{2}:\d{2}$/.test(resourceConfig.wallTime)) {
      newErrors.wallTime = "Wall time must be in HH:MM:SS format";
    }

    // File validation
    const requiredFiles = [".pdb", ".psf", ".prm"];
    const fileExtensions = uploadedFiles.map(f => f.name.split('.').pop()?.toLowerCase());
    
    for (const ext of requiredFiles) {
      if (!fileExtensions.some(fe => fe === ext.substring(1))) {
        newErrors.files = `Missing required file type: ${ext}`;
        break;
      }
    }

    // NAMD validation
    if (!namdConfig.jobName.trim()) {
      newErrors.jobName = "Job name is required";
    }
    if (!namdConfig.simulationSteps || namdConfig.simulationSteps <= 0) {
      newErrors.simulationSteps = "Simulation steps must be a positive number";
    }
    if (!namdConfig.temperature || namdConfig.temperature <= 0) {
      newErrors.temperature = "Temperature must be a positive number";
    }
    if (!namdConfig.timestep || namdConfig.timestep <= 0) {
      newErrors.timestep = "Timestep must be a positive number";
    }
    if (!namdConfig.outputName.trim()) {
      newErrors.outputName = "Output name is required";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async () => {
    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);

    try {
      // Simulate job creation
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      console.log("Creating job with:", {
        resourceConfig,
        uploadedFiles: uploadedFiles.map(f => ({ name: f.name, size: f.size })),
        namdConfig
      });

      onJobCreated();
    } catch (error) {
      console.error("Error creating job:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="p-6 space-y-6">
      {/* Back Button */}
      <Button
        variant="ghost"
        onClick={onCancel}
        className="mb-4"
      >
        <ArrowLeft className="w-4 h-4 mr-2" />
        Back to Jobs
      </Button>

      <h1>Create New Job</h1>

      {/* SLURM Resource Allocation */}
      <Card>
        <CardHeader>
          <CardTitle>SLURM Resource Allocation</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceForm
            config={resourceConfig}
            onChange={setResourceConfig}
            errors={errors}
          />
        </CardContent>
      </Card>

      {/* Input Files */}
      <Card>
        <CardHeader>
          <CardTitle>Input Files</CardTitle>
        </CardHeader>
        <CardContent>
          <FileUploadArea
            uploadedFiles={uploadedFiles}
            onChange={setUploadedFiles}
            error={errors.files}
          />
        </CardContent>
      </Card>

      {/* NAMD Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>NAMD Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <NAMDConfigForm
            config={namdConfig}
            onChange={setNAMDConfig}
            errors={errors}
          />
        </CardContent>
      </Card>

      {/* Actions */}
      <div className="flex items-center gap-3 pt-4">
        <Button
          variant="outline"
          onClick={onCancel}
          disabled={isSubmitting}
        >
          Cancel
        </Button>
        <Button
          onClick={handleSubmit}
          disabled={isSubmitting}
        >
          {isSubmitting ? "Creating Job..." : "Create Job"}
        </Button>
      </div>
    </div>
  );
}