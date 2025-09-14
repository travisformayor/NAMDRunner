import { Job } from "../../App";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { OverviewTab } from "./tabs/OverviewTab";
import { SlurmLogsTab } from "./tabs/SlurmLogsTab";
import { InputFilesTab } from "./tabs/InputFilesTab";
import { OutputFilesTab } from "./tabs/OutputFilesTab";
import { ConfigurationTab } from "./tabs/ConfigurationTab";

interface JobTabsProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  job: Job;
}

export function JobTabs({ activeTab, onTabChange, job }: JobTabsProps) {
  return (
    <Tabs value={activeTab} onValueChange={onTabChange}>
      <TabsList className="grid w-full grid-cols-5">
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="slurm-logs">SLURM Logs</TabsTrigger>
        <TabsTrigger value="input-files">Input Files</TabsTrigger>
        <TabsTrigger value="output-files">Output Files</TabsTrigger>
        <TabsTrigger value="configuration">Configuration</TabsTrigger>
      </TabsList>
      
      <TabsContent value="overview" className="mt-6">
        <OverviewTab job={job} />
      </TabsContent>
      
      <TabsContent value="slurm-logs" className="mt-6">
        <SlurmLogsTab job={job} />
      </TabsContent>
      
      <TabsContent value="input-files" className="mt-6">
        <InputFilesTab job={job} />
      </TabsContent>
      
      <TabsContent value="output-files" className="mt-6">
        <OutputFilesTab job={job} />
      </TabsContent>
      
      <TabsContent value="configuration" className="mt-6">
        <ConfigurationTab job={job} />
      </TabsContent>
    </Tabs>
  );
}