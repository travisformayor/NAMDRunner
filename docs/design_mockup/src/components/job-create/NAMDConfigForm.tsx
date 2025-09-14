import { NAMDConfig } from "./CreateJobView";
import { Input } from "../ui/input";
import { Label } from "../ui/label";

interface NAMDConfigFormProps {
  config: NAMDConfig;
  onChange: (config: NAMDConfig) => void;
  errors: Record<string, string>;
}

export function NAMDConfigForm({ config, onChange, errors }: NAMDConfigFormProps) {
  const updateConfig = (field: keyof NAMDConfig, value: string | number) => {
    onChange({
      ...config,
      [field]: value
    });
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div className="space-y-2">
        <Label htmlFor="jobName">Job Name *</Label>
        <Input
          id="jobName"
          type="text"
          value={config.jobName}
          onChange={(e) => updateConfig("jobName", e.target.value)}
          placeholder="protein_simulation"
        />
        {errors.jobName && (
          <div className="text-sm text-destructive">{errors.jobName}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="outputName">Output Name *</Label>
        <Input
          id="outputName"
          type="text"
          value={config.outputName}
          onChange={(e) => updateConfig("outputName", e.target.value)}
          placeholder="output"
        />
        {errors.outputName && (
          <div className="text-sm text-destructive">{errors.outputName}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="simulationSteps">Simulation Steps *</Label>
        <Input
          id="simulationSteps"
          type="number"
          value={config.simulationSteps}
          onChange={(e) => updateConfig("simulationSteps", parseInt(e.target.value) || 0)}
          min="1"
          placeholder="1000000"
        />
        {errors.simulationSteps && (
          <div className="text-sm text-destructive">{errors.simulationSteps}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="temperature">Temperature (K) *</Label>
        <Input
          id="temperature"
          type="number"
          value={config.temperature}
          onChange={(e) => updateConfig("temperature", parseFloat(e.target.value) || 0)}
          min="0"
          step="0.1"
          placeholder="310"
        />
        {errors.temperature && (
          <div className="text-sm text-destructive">{errors.temperature}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="timestep">Timestep (fs) *</Label>
        <Input
          id="timestep"
          type="number"
          value={config.timestep}
          onChange={(e) => updateConfig("timestep", parseFloat(e.target.value) || 0)}
          min="0"
          step="0.1"
          placeholder="2.0"
        />
        {errors.timestep && (
          <div className="text-sm text-destructive">{errors.timestep}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="dcdFreq">DCD Frequency</Label>
        <Input
          id="dcdFreq"
          type="number"
          value={config.dcdFreq || ""}
          onChange={(e) => updateConfig("dcdFreq", parseInt(e.target.value) || undefined)}
          min="1"
          placeholder="5000"
        />
        <div className="text-xs text-muted-foreground">
          How often to write trajectory frames (steps)
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="restartFreq">Restart Frequency</Label>
        <Input
          id="restartFreq"
          type="number"
          value={config.restartFreq || ""}
          onChange={(e) => updateConfig("restartFreq", parseInt(e.target.value) || undefined)}
          min="1"
          placeholder="10000"
        />
        <div className="text-xs text-muted-foreground">
          How often to write restart files (steps)
        </div>
      </div>
    </div>
  );
}