import { useState } from "react";
import { ConnectionState } from "../../App";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { ChevronDown, Circle } from "lucide-react";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "../ui/popover";

interface ConnectionStatusProps {
  state: ConnectionState;
  onStateChange: (state: ConnectionState) => void;
}

export function ConnectionStatus({ state, onStateChange }: ConnectionStatusProps) {
  const [host, setHost] = useState("cluster.edu");
  const [username, setUsername] = useState("jsmith");
  const [password, setPassword] = useState("");

  const getStatusInfo = () => {
    switch (state) {
      case "connected":
        return { 
          label: "Connected", 
          color: "text-green-600", 
          dotColor: "fill-green-600",
          since: "10:30 AM"
        };
      case "connecting":
        return { 
          label: "Connecting...", 
          color: "text-yellow-600", 
          dotColor: "fill-yellow-600" 
        };
      case "disconnected":
        return { 
          label: "Disconnected", 
          color: "text-red-600", 
          dotColor: "fill-red-600" 
        };
      case "expired":
        return { 
          label: "Connection Expired", 
          color: "text-gray-600", 
          dotColor: "fill-gray-600" 
        };
    }
  };

  const statusInfo = getStatusInfo();

  const handleConnect = () => {
    if (host && username && password) {
      onStateChange("connecting");
      // Simulate connection
      setTimeout(() => {
        onStateChange("connected");
      }, 2000);
    }
  };

  const handleDisconnect = () => {
    onStateChange("disconnected");
    setPassword("");
  };

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button 
          variant="ghost" 
          className="flex items-center gap-2"
          data-testid="connection-status-button"
        >
          <Circle className={`w-2 h-2 ${statusInfo.dotColor}`} />
          <span className={statusInfo.color}>{statusInfo.label}</span>
          <ChevronDown className="w-4 h-4" />
        </Button>
      </PopoverTrigger>
      
      <PopoverContent 
        align="end" 
        className="w-64 p-4" 
        sideOffset={8}
      >
        {state === "connected" ? (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Circle className="w-2 h-2 fill-green-600" />
              <span className="text-green-600">Connected</span>
            </div>
            <div className="space-y-1 text-sm">
              <div>Host: {host}</div>
              <div>User: {username}</div>
              {statusInfo.since && <div>Since: {statusInfo.since}</div>}
            </div>
            <Button 
              variant="outline" 
              size="sm" 
              onClick={handleDisconnect}
              className="w-full"
            >
              Disconnect
            </Button>
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Circle className={`w-2 h-2 ${statusInfo.dotColor}`} />
              <span className={statusInfo.color}>{statusInfo.label}</span>
            </div>
            
            <div className="space-y-3">
              <div>
                <Label htmlFor="host">Host</Label>
                <Input
                  id="host"
                  value={host}
                  onChange={(e) => setHost(e.target.value)}
                  placeholder="cluster.edu"
                />
              </div>
              
              <div>
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="username"
                />
              </div>
              
              <div>
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="password"
                />
              </div>
              
              <Button 
                onClick={handleConnect}
                disabled={!host || !username || !password || state === "connecting"}
                className="w-full"
              >
                {state === "connecting" ? "Connecting..." : "Connect"}
              </Button>
            </div>
          </div>
        )}
      </PopoverContent>
    </Popover>
  );
}