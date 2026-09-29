import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";

/**
 * Section de formulaire modal : sur-titre `caps` du design (`DESIGN.md` §4), puis le
 * contenu. Les modales entreprise / contact / entretien / relance partagent cette
 * structure ; un `legend` maison (`text-eyebrow`…) divergait du reste de l'app.
 */
export function FormSection({
  title,
  className,
  children,
}: {
  title: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <fieldset className={cn("min-w-0", className)}>
      <legend className="caps mb-3 px-0">{title}</legend>
      {children}
    </fieldset>
  );
}
