import { Job } from "../../../App";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";
import { Badge } from "../../ui/badge";

interface ConfigurationTabProps {
  job: Job;
}

// Mock configuration data
const namdConfig = {
  simulationSteps: 1000000,
  temperature: 310.0,
  timestep: 2.0,
  outputName: "protein_simulation",
  dcdFreq: 5000,
  restartFreq: 10000,
  coordinates: "protein.pdb",
  structure: "protein.psf",
  parameters: ["par_all36_prot.prm", "par_all36_lipid.prm", "toppar_water_ions.str"],
  cutoff: 12.0,
  switchDist: 10.0,
  pairlistDist: 14.0,
  PME: true,
  PMEGridSpacing: 1.0,
  langevin: true,
  langevinDamping: 1.0,
  langevinTemp: 310.0,
  langevinHydrogen: false,
  useGroupPressure: true,
  useFlexibleCell: false,
  useConstantArea: false,
  langevinPiston: true,
  langevinPistonTarget: 1.01325,
  langevinPistonPeriod: 100.0,
  langevinPistonDecay: 50.0,
  langevinPistonTemp: 310.0
};

const slurmConfig = {
  cores: 128,
  memory: "512GB",
  wallTime: "04:00:00",
  partition: "amilan",
  qos: "normal",
  account: "research",
  nodes: 4,
  tasksPerNode: 32,
  cpusPerTask: 1,
  gpus: 4,
  gpuType: "v100"
};

export function ConfigurationTab({ job }: ConfigurationTabProps) {
  return (
    <div className="space-y-6">
      {/* NAMD Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>NAMD Configuration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Basic Settings */}
          <div>
            <h4 className="font-medium mb-3">Basic Settings</h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
              <div>
                <div className="text-muted-foreground">Simulation Steps</div>
                <div className="font-mono">{namdConfig.simulationSteps.toLocaleString()}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Temperature (K)</div>
                <div className="font-mono">{namdConfig.temperature}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Timestep (fs)</div>
                <div className="font-mono">{namdConfig.timestep}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Output Name</div>
                <div className="font-mono">{namdConfig.outputName}</div>
              </div>
              <div>
                <div className="text-muted-foreground">DCD Frequency</div>
                <div className="font-mono">{namdConfig.dcdFreq.toLocaleString()}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Restart Frequency</div>
                <div className="font-mono">{namdConfig.restartFreq.toLocaleString()}</div>
              </div>
            </div>
          </div>

          {/* Input Files */}
          <div>
            <h4 className="font-medium mb-3">Input Files</h4>
            <div className="space-y-2 text-sm">
              <div>
                <span className="text-muted-foreground">Coordinates:</span> 
                <span className="ml-2 font-mono">{namdConfig.coordinates}</span>
              </div>
              <div>
                <span className="text-muted-foreground">Structure:</span> 
                <span className="ml-2 font-mono">{namdConfig.structure}</span>
              </div>
              <div>
                <span className="text-muted-foreground">Parameters:</span>
                <div className="ml-2 mt-1 flex flex-wrap gap-2">
                  {namdConfig.parameters.map((param, index) => (
                    <Badge key={index} variant="secondary" className="font-mono text-xs">
                      {param}
                    </Badge>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Force Field & Cutoffs */}
          <div>
            <h4 className="font-medium mb-3">Force Field & Cutoffs</h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
              <div>
                <div className="text-muted-foreground">Cutoff (Å)</div>
                <div className="font-mono">{namdConfig.cutoff}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Switch Distance (Å)</div>
                <div className="font-mono">{namdConfig.switchDist}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Pairlist Distance (Å)</div>
                <div className="font-mono">{namdConfig.pairlistDist}</div>
              </div>
              <div>
                <div className="text-muted-foreground">PME</div>
                <Badge variant={namdConfig.PME ? "default" : "secondary"}>
                  {namdConfig.PME ? "Enabled" : "Disabled"}
                </Badge>
              </div>
              <div>
                <div className="text-muted-foreground">PME Grid Spacing (Å)</div>
                <div className="font-mono">{namdConfig.PMEGridSpacing}</div>
              </div>
            </div>
          </div>

          {/* Dynamics Settings */}
          <div>
            <h4 className="font-medium mb-3">Dynamics Settings</h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
              <div>
                <div className="text-muted-foreground">Langevin Dynamics</div>
                <Badge variant={namdConfig.langevin ? "default" : "secondary"}>
                  {namdConfig.langevin ? "Enabled" : "Disabled"}
                </Badge>
              </div>
              <div>
                <div className="text-muted-foreground">Langevin Damping</div>
                <div className="font-mono">{namdConfig.langevinDamping} ps⁻¹</div>
              </div>
              <div>
                <div className="text-muted-foreground">Langevin Temperature (K)</div>
                <div className="font-mono">{namdConfig.langevinTemp}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Langevin Hydrogen</div>
                <Badge variant={namdConfig.langevinHydrogen ? "default" : "secondary"}>
                  {namdConfig.langevinHydrogen ? "Enabled" : "Disabled"}
                </Badge>
              </div>
            </div>
          </div>

          {/* Pressure Control */}
          <div>
            <h4 className="font-medium mb-3">Pressure Control</h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
              <div>
                <div className="text-muted-foreground">Langevin Piston</div>
                <Badge variant={namdConfig.langevinPiston ? "default" : "secondary"}>
                  {namdConfig.langevinPiston ? "Enabled" : "Disabled"}
                </Badge>
              </div>
              <div>
                <div className="text-muted-foreground">Target Pressure (bar)</div>
                <div className="font-mono">{namdConfig.langevinPistonTarget}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Piston Period (fs)</div>
                <div className="font-mono">{namdConfig.langevinPistonPeriod}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Piston Decay (fs)</div>
                <div className="font-mono">{namdConfig.langevinPistonDecay}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Piston Temperature (K)</div>
                <div className="font-mono">{namdConfig.langevinPistonTemp}</div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* SLURM Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>SLURM Resource Allocation</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
            <div>
              <div className="text-muted-foreground">Cores</div>
              <div className="font-mono">{slurmConfig.cores}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Memory</div>
              <div className="font-mono">{slurmConfig.memory}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Wall Time</div>
              <div className="font-mono">{slurmConfig.wallTime}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Partition</div>
              <div className="font-mono">{slurmConfig.partition}</div>
            </div>
            <div>
              <div className="text-muted-foreground">QOS</div>
              <div className="font-mono">{slurmConfig.qos}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Account</div>
              <div className="font-mono">{slurmConfig.account}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Nodes</div>
              <div className="font-mono">{slurmConfig.nodes}</div>
            </div>
            <div>
              <div className="text-muted-foreground">Tasks per Node</div>
              <div className="font-mono">{slurmConfig.tasksPerNode}</div>
            </div>
            <div>
              <div className="text-muted-foreground">CPUs per Task</div>
              <div className="font-mono">{slurmConfig.cpusPerTask}</div>
            </div>
            <div>
              <div className="text-muted-foreground">GPUs</div>
              <div className="font-mono">{slurmConfig.gpus} × {slurmConfig.gpuType.toUpperCase()}</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}