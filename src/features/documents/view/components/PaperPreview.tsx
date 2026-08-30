import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";

/** Feuille d'aperçu imprimable, alimentée uniquement par les jetons papier globaux. */
export function PaperPreview({
  title,
  children,
  className,
}: {
  title: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <article aria-label={title} className={cn("paper-preview", className)}>
      {children}
    </article>
  );
}
