import * as React from "react";
import { cn } from "@/lib/utils";

export const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, ...props }, ref) => (
    <input
      ref={ref}
      className={cn(
        "h-9 rounded-md border border-border bg-background/40 px-3 text-sm placeholder:text-muted-foreground transition-colors hover:bg-background/60 focus:bg-background/60 focus:outline-none focus:ring-2 focus:ring-ring/40",
        className,
      )}
      {...props}
    />
  ),
);
Input.displayName = "Input";
