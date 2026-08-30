import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/** Sur-titre mono en capitales — le motif d'étiquette de section du design :
 *  10,5px / 600, interlettrage 0.07em, `--ink-faint`. */
export function EtiquetteMono({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <p
      className={cn(
        "font-mono text-[10.5px] font-semibold uppercase tracking-[0.07em] text-ink-faint",
        className,
      )}
    >
      {children}
    </p>
  );
}
