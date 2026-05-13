import { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface RowProps {
  label: ReactNode;
  desc?: ReactNode;
  children?: ReactNode;
  className?: string;
}

export function Row({ label, desc, children, className }: RowProps) {
  return (
    <div
      className={cn(
        "flex items-center justify-between gap-4 border-b border-border/40 py-4 last:border-b-0",
        className,
      )}
    >
      <div className="min-w-0">
        <div className="text-sm font-medium">{label}</div>
        {desc && (
          <div className="mt-0.5 text-xs text-muted-foreground">{desc}</div>
        )}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

export function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <div className="mb-3 mt-6 text-xs font-semibold uppercase tracking-wider text-muted-foreground first:mt-0">
      {children}
    </div>
  );
}
