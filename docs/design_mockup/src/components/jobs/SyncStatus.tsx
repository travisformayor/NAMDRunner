import { ConnectionState } from "../../App";
import { Button } from "../ui/button";
import { Checkbox } from "../ui/checkbox";
import { Input } from "../ui/input";
import { RefreshCw } from "lucide-react";

interface SyncStatusProps {
  lastSyncTime: Date;
  autoSync: boolean;
  syncInterval: number;
  connectionState: ConnectionState;
  onSyncNow: () => void;
  onAutoSyncChange: (enabled: boolean) => void;
  onSyncIntervalChange: (interval: number) => void;
}

export function SyncStatus({
  lastSyncTime,
  autoSync,
  syncInterval,
  connectionState,
  onSyncNow,
  onAutoSyncChange,
  onSyncIntervalChange
}: SyncStatusProps) {
  const formatLastSync = (date: Date) => {
    const now = new Date();
    const diffMinutes = Math.floor((now.getTime() - date.getTime()) / (1000 * 60));
    
    if (diffMinutes < 1) return "Just now";
    if (diffMinutes === 1) return "1 minute ago";
    if (diffMinutes < 60) return `${diffMinutes} minutes ago`;
    
    const diffHours = Math.floor(diffMinutes / 60);
    if (diffHours === 1) return "1 hour ago";
    return `${diffHours} hours ago`;
  };

  const getStatusText = () => {
    if (connectionState === "connected") {
      return `Last synced: ${formatLastSync(lastSyncTime)}`;
    } else {
      return `Offline - showing cached data from ${lastSyncTime.toLocaleString()}`;
    }
  };

  return (
    <div className="flex items-center justify-between text-sm text-muted-foreground">
      <div className="flex items-center gap-4">
        <span className={connectionState !== "connected" ? "text-muted-foreground" : ""}>
          {getStatusText()}
        </span>
        
        <Button
          variant="ghost"
          size="sm"
          onClick={onSyncNow}
          disabled={connectionState !== "connected"}
          className="h-8 px-2 text-sm"
        >
          <RefreshCw className="w-3 h-3 mr-1" />
          Sync Now
        </Button>
      </div>

      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <Checkbox
            id="auto-sync"
            checked={autoSync}
            onCheckedChange={(checked) => onAutoSyncChange(checked === true)}
            disabled={connectionState !== "connected"}
          />
          <label htmlFor="auto-sync" className="text-sm">
            Auto-sync: every
          </label>
        </div>
        
        <Input
          type="number"
          value={syncInterval}
          onChange={(e) => onSyncIntervalChange(parseInt(e.target.value) || 5)}
          disabled={!autoSync || connectionState !== "connected"}
          className="w-16 h-8 text-sm"
          min="1"
          max="60"
        />
        
        <span className="text-sm">min</span>
      </div>
    </div>
  );
}