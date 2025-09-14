import { View } from "../../App";
import { Button } from "../ui/button";
import { Briefcase, Plus, Settings } from "lucide-react";

interface SidebarProps {
  currentView: View;
  onViewChange: (view: View) => void;
}

export function Sidebar({ currentView, onViewChange }: SidebarProps) {
  const menuItems = [
    {
      id: "jobs" as const,
      label: "Jobs",
      icon: Briefcase,
      active: currentView === "jobs"
    },
    {
      id: "create" as const,
      label: "Create Job", 
      icon: Plus,
      active: currentView === "create"
    },
    {
      id: "settings" as const,
      label: "Settings",
      icon: Settings,
      active: currentView === "settings",
      disabled: true // Future enhancement
    }
  ];

  return (
    <div className="w-48 bg-sidebar border-r border-sidebar-border flex flex-col">
      <div className="p-4">
        <h1 className="text-xl font-medium text-sidebar-foreground">NAMDRunner</h1>
      </div>
      
      <nav className="flex-1 px-2">
        {menuItems.map((item) => {
          const Icon = item.icon;
          return (
            <Button
              key={item.id}
              variant={item.active ? "default" : "ghost"}
              className={`w-full justify-start mb-1 ${
                item.active 
                  ? "bg-sidebar-primary text-sidebar-primary-foreground" 
                  : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              } ${item.disabled ? "opacity-50 cursor-not-allowed" : ""}`}
              onClick={() => !item.disabled && onViewChange(item.id)}
              disabled={item.disabled}
            >
              <Icon className="w-4 h-4 mr-3" />
              {item.label}
            </Button>
          );
        })}
      </nav>
    </div>
  );
}