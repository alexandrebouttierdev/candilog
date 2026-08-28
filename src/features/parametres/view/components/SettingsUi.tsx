import type { ReactNode } from "react";
import { Icon } from "@/shared/ui";

/** Contenu défilant des écrans Réglages : 22 px / 28 px, comme les maquettes. */
export function SettingsBody({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="flex flex-col gap-4 px-7 pt-[22px] pb-[34px]">{children}</div>
    </div>
  );
}

/**
 * Carte de section des Réglages : en-tête à filet 14 px / 18 px, icône tertiaire, titre 13 px.
 */
export function SettingsCard({
  icon,
  title,
  hint,
  className,
  children,
}: {
  icon: string;
  title: string;
  hint?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <section
      className={
        className ??
        "min-w-0 overflow-hidden rounded-card border border-line bg-surface shadow-e1"
      }
    >
      <div className="border-b border-line px-[18px] py-[14px]">
        <div className="flex items-center gap-2">
          <Icon name={icon} size={17} className="flex-none text-ink-faint" />
          <h2 className="text-item font-semibold text-ink">{title}</h2>
        </div>
        {hint ? <p className="mt-1 ml-[25px] text-label text-ink-faint">{hint}</p> : null}
      </div>
      <div className="px-[18px] py-4">{children}</div>
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
    <section className="w-full min-w-0 rounded-card border border-line bg-surface px-[18px] py-5 shadow-e1">
      <p className="text-eyebrow text-accent uppercase">{kicker}</p>
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
    <section className="flex min-h-44 min-w-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface shadow-e1">
      <div className="flex items-center gap-2 border-b border-line px-[18px] py-[14px]">
        <span className="flex size-[26px] items-center justify-center rounded-control bg-accent-tint text-accent">
          <Icon name={icon} size={15} />
        </span>
        <h2 className="text-item font-semibold text-ink">{title}</h2>
      </div>
      <div className="flex flex-1 flex-col px-[18px] py-4">
        <p className="mb-4 flex-1 text-body leading-relaxed text-ink-muted">{description}</p>
        {children}
      </div>
    </section>
  );
}
