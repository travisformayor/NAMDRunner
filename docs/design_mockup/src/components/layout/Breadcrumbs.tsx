import { ChevronRight } from "lucide-react";
import { Button } from "../ui/button";

interface BreadcrumbItem {
  label: string;
  onClick?: () => void;
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
}

export function Breadcrumbs({ items }: BreadcrumbsProps) {
  return (
    <nav className="flex items-center space-x-1">
      {items.map((item, index) => (
        <div key={index} className="flex items-center">
          {index > 0 && (
            <ChevronRight className="w-4 h-4 text-muted-foreground mx-1" />
          )}
          {item.onClick ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={item.onClick}
              className="h-auto p-1 text-sm hover:bg-accent"
            >
              {item.label}
            </Button>
          ) : (
            <span className="text-sm text-foreground font-medium px-1">
              {item.label}
            </span>
          )}
        </div>
      ))}
    </nav>
  );
}