import type { ReactNode } from "react";
import { Icon } from "@/shared/ui";

/** Contenu défilant des écrans Réglages, largeur lisible. */
export function SettingsBody({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto max-w-4xl space-y-4 p-5 min-[1200px]:p-6">{children}</div>
    </div>
  );
}

/** Carte de section avec icône et titre. */
export function SettingsCard({
  icon,
  title,
  children,
}: {
  icon: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-card border border-line bg-surface p-5 shadow-e1">
      <div className="mb-4 flex items-center gap-2">
        <Icon name={icon} size={18} className="text-accent" />
        <h2 className="text-section text-ink">{title}</h2>
      </div>
      {children}
    </section>
  );
}

/** Accroche en tête d'écran de maintenance. */
export function SettingsHero({
  kicker,
  title,
  description,
}: {
  kicker: string;
  title: string;
  description: string;
}) {
  return (
    <section className="rounded-card border border-line bg-surface p-6 shadow-e1">
      <p className="text-meta font-semibold tracking-wide text-accent uppercase">{kicker}</p>
      <h2 className="mt-1 text-title text-ink">{title}</h2>
      <p className="mt-2 max-w-2xl text-body leading-relaxed text-ink-muted">{description}</p>
    </section>
  );
}

/** Carte d'action (export, restauration, disponibilité). */
export function ActionCard({
  icon,
  title,
  description,
  children,
}: {
  icon: string;
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <section className="flex min-h-44 flex-col rounded-card border border-line bg-surface p-5 shadow-e1">
      <div className="mb-3 flex items-center gap-2">
        <span className="flex size-8 items-center justify-center rounded-card bg-accent-tint text-accent">
          <Icon name={icon} size={16} />
        </span>
        <h2 className="text-section text-ink">{title}</h2>
      </div>
      <p className="mb-4 flex-1 text-body leading-relaxed text-ink-muted">{description}</p>
      {children}
    </section>
  );
}
