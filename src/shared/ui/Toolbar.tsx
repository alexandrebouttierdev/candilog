import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";

/**
 * Barre d'outils compacte d'un workspace.
 */
export function Toolbar({
  title,
  subtitle,
  left,
  right,
  className,
}: {
  title?: string;
  subtitle?: string;
  left?: ReactNode;
  right?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex h-topbar flex-none items-center gap-3 border-b border-line-soft px-4",
        className,
      )}
    >
      {title ? (
        <div className="flex min-w-0 items-center gap-2">
          <h1 className="truncate text-section text-ink">{title}</h1>
          {subtitle ? (
            <>
              <span aria-hidden className="h-3.5 w-px flex-none bg-line" />
              <p className="truncate text-note text-ink-faint">{subtitle}</p>
            </>
          ) : null}
        </div>
      ) : null}
      {left}
      <span className="flex-1" />
      {right}
    </div>
  );
}
