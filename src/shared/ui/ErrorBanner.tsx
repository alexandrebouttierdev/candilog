import type { ReactNode } from "react";
import { StatusGlyph } from "./StatusGlyph";
import { Button } from "./Button";
import { cn } from "@/shared/lib/cn";

/**
 * Bandeau d'erreur non bloquant.
 *
 * Encadré des maquettes : filet en `danger-border`, fond en `danger-tint`, rayon 10 px,
 * titre 12,5 px/600 en rouge et détail en gris moyen.
 *
 * Le guide demande une erreur *dans* l'écran plutôt qu'à la place de l'écran : ce qui a pu
 * être chargé reste visible, et l'utilisateur garde une action — « Réessayer » — au lieu
 * d'une page morte.
 */
export function ErrorBanner({
  title = "Chargement impossible",
  message,
  onRetry,
}: {
  title?: string;
  message: string;
  onRetry?: () => void;
}) {
  return (
    <Banner tone="danger" title={title} message={message}>
      {onRetry ? (
        <Button variant="secondary" onClick={onRetry}>
          Réessayer
        </Button>
      ) : null}
    </Banner>
  );
}

/** Glyphe de statut du bandeau : la teinte seule ne porte jamais l'information. */
const GLYPHS = { danger: "c", success: "g", accent: "n" } as const;

/** Bandeau d'information, de succès ou d'erreur, aux trois teintes des maquettes. */
export function Banner({
  tone,
  title,
  message,
  children,
  className,
}: {
  tone: "danger" | "success" | "accent";
  title: string;
  message?: ReactNode;
  children?: ReactNode;
  className?: string;
}) {
  const TONES = {
    danger: "border-danger-border bg-danger-tint text-danger",
    success: "border-success-border bg-success-tint text-success",
    accent: "border-accent-border bg-accent-tint text-accent",
  } as const;

  return (
    <div
      role={tone === "danger" ? "alert" : "status"}
      className={cn(
        "flex items-start gap-[11px] rounded-tile border p-[13px]",
        TONES[tone],
        className,
      )}
    >
      <span className="mt-[3px] flex-none">
        <StatusGlyph tone={GLYPHS[tone]} />
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-body font-semibold">{title}</p>
        {message ? (
          <p className="mt-[3px] text-label leading-normal text-ink-muted">{message}</p>
        ) : null}
      </div>
      {children ? <div className="flex flex-none items-center gap-2">{children}</div> : null}
    </div>
  );
}
