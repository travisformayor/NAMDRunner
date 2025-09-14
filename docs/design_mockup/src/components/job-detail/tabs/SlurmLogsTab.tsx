import { useState } from "react";
import { Job } from "../../../App";
import { Button } from "../../ui/button";
import { ScrollArea } from "../../ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../../ui/tabs";
import { Copy, Download } from "lucide-react";

interface SlurmLogsTabProps {
  job: Job;
}

const mockStdout = `NAMD 3.0beta3 for LINUX-X86_64-multicore
Built Thu Aug 26 16:49:51 CDT 2021 by jim on belfast.ks.uiuc.edu

Running on 128 processors.
CHARM> Running on 128 cores
CHARM> Number of chares: 128

Info: NAMD 3.0beta3 for LINUX-X86_64-multicore
Info: Built Thu Aug 26 16:49:51 CDT 2021 by jim on belfast.ks.uiuc.edu
Info: 1 CUDA devices on 1 nodes
Info: CUDA device 0 is Tesla V100-SXM2-16GB

Info: Running on 128 processors, 128 cores, 1 hosts.
Info: CPU topology information available.
Info: Charm++/Converse parallel runtime startup completed at 0.0123 s
Info: NAMD running on 128 processors, 128 cores, 1 hosts.

Reading structure file protein.psf
protein.psf info: 92428 atoms, 7324 bonds, 13251 angles, 18746 dihedrals,
                  11874 impropers, 0 cross-terms

Reading parameter file par_all36_prot.prm
Reading parameter file par_all36_lipid.prm  
Reading parameter file toppar_water_ions.str

Info: READING COORDINATES FROM DCD FILE protein_eq.dcd
Info: DCD file protein_eq.dcd at time 0.0

TIMING: 500  CPU: 12.45, 0.0249/step  Wall: 12.45, 0.0249/step, 34.57 hours remaining
TIMING: 1000  CPU: 24.89, 0.0249/step  Wall: 24.89, 0.0249/step, 34.56 hours remaining
ENERGY:    1000      7892.3456   23451.2341    -89234.5678       234.5671    -45612.3456     8234.5672       0.0000       0.0000       0.0000  -95834.2034

TIMING: 1500  CPU: 37.34, 0.0249/step  Wall: 37.34, 0.0249/step, 34.55 hours remaining
TIMING: 2000  CPU: 49.78, 0.0249/step  Wall: 49.78, 0.0249/step, 34.54 hours remaining

[... continuing simulation ...]

TIMING: 450000  CPU: 11234.56, 0.0249/step  Wall: 11234.56, 0.0249/step, 1.23 hours remaining
ENERGY:  450000     7834.2341   23892.1234    -89567.8901       245.6789    -45823.4567     8456.7890       0.0000       0.0000       0.0000  -95962.5214

TIMING: 450500  CPU: 11247.01, 0.0249/step  Wall: 11247.01, 0.0249/step, 1.22 hours remaining`;

const mockStderr = `Info: Startup phase 0 took 0.00123 s, 128 KB of memory in use
Info: Startup phase 1 took 0.00456 s, 256 KB of memory in use
Info: Startup phase 2 took 0.00789 s, 512 KB of memory in use
Info: Startup phase 3 took 0.01234 s, 1024 KB of memory in use
Info: Startup phase 4 took 0.02345 s, 2048 KB of memory in use
Info: Startup phase 5 took 0.03456 s, 4096 KB of memory in use

Warning: Continuing with simulation despite potential energy instability
Warning: Large force detected at step 12345, atom 67890
Warning: Temperature spike detected at step 23456, T = 315.67 K

Info: Checkpoint written at step 100000
Info: Checkpoint written at step 200000
Info: Checkpoint written at step 300000
Info: Checkpoint written at step 400000

Warning: GPU memory usage approaching limit: 92%
Info: Automatic load balancing triggered at step 445000
Info: Load balancing completed, performance improved by 3.2%`;

export function SlurmLogsTab({ job }: SlurmLogsTabProps) {
  const [activeLog, setActiveLog] = useState("stdout");

  const handleCopy = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  const handleDownload = (content: string, filename: string) => {
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const getCurrentContent = () => {
    return activeLog === "stdout" ? mockStdout : mockStderr;
  };

  const getCurrentFilename = () => {
    return activeLog === "stdout" ? `${job.jobId}_stdout.log` : `${job.jobId}_stderr.log`;
  };

  if (job.status === "CREATED") {
    return (
      <div className="text-center py-8 text-muted-foreground">
        Logs will be available once the job starts running.
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <Tabs value={activeLog} onValueChange={setActiveLog}>
        <div className="flex items-center justify-between">
          <TabsList>
            <TabsTrigger value="stdout">Standard Output</TabsTrigger>
            <TabsTrigger value="stderr">Standard Error</TabsTrigger>
          </TabsList>
          
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleCopy(getCurrentContent())}
            >
              <Copy className="w-4 h-4 mr-2" />
              Copy
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleDownload(getCurrentContent(), getCurrentFilename())}
            >
              <Download className="w-4 h-4 mr-2" />
              Download
            </Button>
          </div>
        </div>

        <TabsContent value="stdout" className="mt-4">
          <div className="border border-border rounded-lg bg-muted/30">
            <ScrollArea className="h-96 p-4">
              <pre className="text-xs font-mono whitespace-pre-wrap">
                {mockStdout}
              </pre>
            </ScrollArea>
          </div>
        </TabsContent>

        <TabsContent value="stderr" className="mt-4">
          <div className="border border-border rounded-lg bg-muted/30">
            <ScrollArea className="h-96 p-4">
              <pre className="text-xs font-mono whitespace-pre-wrap">
                {mockStderr}
              </pre>
            </ScrollArea>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}