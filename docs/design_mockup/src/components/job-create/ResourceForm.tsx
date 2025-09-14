import { ResourceConfig } from "./CreateJobView";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";

interface ResourceFormProps {
  config: ResourceConfig;
  onChange: (config: ResourceConfig) => void;
  errors: Record<string, string>;
}

export function ResourceForm({ config, onChange, errors }: ResourceFormProps) {
  const updateConfig = (field: keyof ResourceConfig, value: string | number) => {
    onChange({
      ...config,
      [field]: value
    });
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div className="space-y-2">
        <Label htmlFor="cores">Cores *</Label>
        <Input
          id="cores"
          type="number"
          value={config.cores}
          onChange={(e) => updateConfig("cores", parseInt(e.target.value) || 0)}
          min="1"
          max="1024"
          placeholder="128"
        />
        {errors.cores && (
          <div className="text-sm text-destructive">{errors.cores}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="memory">Memory (GB) *</Label>
        <Input
          id="memory"
          type="text"
          value={config.memory}
          onChange={(e) => updateConfig("memory", e.target.value)}
          placeholder="512"
        />
        {errors.memory && (
          <div className="text-sm text-destructive">{errors.memory}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="wallTime">Wall Time *</Label>
        <Input
          id="wallTime"
          type="text"
          value={config.wallTime}
          onChange={(e) => updateConfig("wallTime", e.target.value)}
          placeholder="HH:MM:SS"
        />
        <div className="text-xs text-muted-foreground">
          Format: HH:MM:SS (e.g., 04:00:00 for 4 hours)
        </div>
        {errors.wallTime && (
          <div className="text-sm text-destructive">{errors.wallTime}</div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="partition">Partition</Label>
        <Select
          value={config.partition}
          onValueChange={(value) => updateConfig("partition", value)}
        >
          <SelectTrigger id="partition">
            <SelectValue placeholder="Select partition" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="amilan">amilan</SelectItem>
            <SelectItem value="acomputeq">acomputeq</SelectItem>
            <SelectItem value="agpu">agpu</SelectItem>
            <SelectItem value="ahigh">ahigh</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="qos">QOS</Label>
        <Select
          value={config.qos}
          onValueChange={(value) => updateConfig("qos", value)}
        >
          <SelectTrigger id="qos">
            <SelectValue placeholder="Select QOS" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="normal">normal</SelectItem>
            <SelectItem value="preempt">preempt</SelectItem>
            <SelectItem value="high">high</SelectItem>
            <SelectItem value="testing">testing</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}