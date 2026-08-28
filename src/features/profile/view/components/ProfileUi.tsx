import type { KeyboardEvent, ReactNode } from "react";
import type { Identity } from "@/shared/types/generated/profile";
import { cn } from "@/shared/lib/cn";
import { Icon, Skeleton } from "@/shared/ui";

export type ProfileTab = "experiences" | "skills" | "education" | "languages";

const TAB_LABELS: Record<ProfileTab, { label: string; icon: string }> = {
  experiences: { label: "Expériences", icon: "work_history" },
  skills: { label: "Compétences", icon: "psychology" },
  education: { label: "Formations", icon: "school" },
  languages: { label: "Langues", icon: "translate" },
};

/** Onglets du bandeau profil : 9 px / 13 px, soulignement accent, compteur 10,5 px. */
export function ProfileTabs({
  active,
  counts,
  onChange,
}: {
  active: ProfileTab;
  counts: Record<ProfileTab, number>;
  onChange: (tab: ProfileTab) => void;
}) {
  return (
    <div role="tablist" aria-label="Sections du profil" className="mt-[18px] flex gap-[3px]">
      {(Object.keys(TAB_LABELS) as ProfileTab[]).map((tab) => {
        const meta = TAB_LABELS[tab];
        const tabs = Object.keys(TAB_LABELS) as ProfileTab[];
        const index = tabs.indexOf(tab);
        const naviguer = (event: KeyboardEvent<HTMLButtonElement>) => {
          let prochain = index;
          if (event.key === "ArrowRight") prochain = (index + 1) % tabs.length;
          else if (event.key === "ArrowLeft") prochain = (index - 1 + tabs.length) % tabs.length;
          else if (event.key === "Home") prochain = 0;
          else if (event.key === "End") prochain = tabs.length - 1;
          else return;
          event.preventDefault();
          const prochainTab = tabs[prochain];
          if (!prochainTab) return;
          onChange(prochainTab);
          document.getElementById(`profil-tab-${prochainTab}`)?.focus();
        };
        const selected = active === tab;
        return (
          <button
            key={tab}
            id={`profil-tab-${tab}`}
            type="button"
            role="tab"
            aria-selected={selected}
            aria-controls={`profil-panel-${tab}`}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(tab)}
            onKeyDown={naviguer}
            className={cn(
              "flex flex-none items-center gap-[7px] px-[13px] py-[9px] text-body font-mid",
              selected ? "text-accent shadow-[inset_0_-2px_0_0_var(--color-accent)]" : "text-ink-muted hover:text-ink",
            )}
          >
            <Icon name={meta.icon} size={16} />
            {meta.label}
            <span className="rounded-tag bg-neutral-tint px-[5px] py-px text-eyebrow font-semibold text-ink-faint">
              {counts[tab]}
            </span>
          </button>
        );
      })}
    </div>
  );
}

export function ProfilePanel({
  tab,
  active,
  children,
}: {
  tab: ProfileTab;
  active: boolean;
  children: ReactNode;
}) {
  if (!active) return null;
  return (
    <div id={`profil-panel-${tab}`} role="tabpanel" aria-labelledby={`profil-tab-${tab}`}>
      {children}
    </div>
  );
}

export function SectionCard({
  icon,
  title,
  actionLabel = "Modifier",
  onEdit,
  children,
}: {
  icon: string;
  title: string;
  actionLabel?: string;
  onEdit: () => void;
  children: ReactNode;
}) {
  return (
    <section className="overflow-hidden rounded-card border border-line bg-surface shadow-e1">
      <header className="flex items-center justify-between gap-3 border-b border-line px-[18px] py-[13px]">
        <div className="flex min-w-0 items-center gap-2">
          <Icon name={icon} size={17} className="flex-none text-ink-faint" />
          <h2 className="truncate text-item font-semibold text-ink">{title}</h2>
        </div>
        <button
          type="button"
          onClick={onEdit}
          className="inline-flex flex-none items-center gap-[5px] rounded-pill text-label font-medium text-accent hover:opacity-80"
        >
          <Icon name="edit" size={15} />
          {actionLabel}
        </button>
      </header>
      {children}
    </section>
  );
}

export function ProfileIdentity({ identity }: { identity: Identity }) {
  const name = [identity.first_name, identity.name].filter(Boolean).join(" ") || "Profil à compléter";
  const initials = [identity.first_name, identity.name]
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase())
    .join("") || "?";
  const chips = [
    identity.email ? { icon: "mail", label: identity.email } : null,
    identity.phone ? { icon: "call", label: identity.phone } : null,
    identity.city ? { icon: "location_on", label: identity.city } : null,
  ].filter((chip): chip is { icon: string; label: string } => chip !== null);

  return (
    <div className="flex min-w-[260px] flex-1 items-start gap-4">
      <span className="flex size-14 flex-none items-center justify-center rounded-full bg-accent-tint text-lg font-strong text-accent">
        {initials}
      </span>
      <div className="min-w-0">
        <h2 className="text-title text-ink">{name}</h2>
        <p className="mt-1 text-body font-mid text-accent">
          {identity.title ?? "Ajoutez votre objectif professionnel"}
        </p>
        {chips.length > 0 ? (
          <div className="mt-2.5 flex flex-wrap gap-2">
            {chips.map((chip) => (
              <span
                key={chip.label}
                className="inline-flex items-center gap-1.5 rounded-pill bg-neutral-tint px-[9px] py-1 text-label text-ink-muted"
              >
                <Icon name={chip.icon} size={14} className="text-ink-faint" />
                {chip.label}
              </span>
            ))}
          </div>
        ) : null}
      </div>
    </div>
  );
}

/** Barre de progression du bandeau, 7 px, comme les maquettes Analytics et Profile. */
export function CompletionBar({
  value,
  hint,
}: {
  value: number;
  hint: string;
}) {
  const borne = Math.max(0, Math.min(100, value));
  return (
    <div className="min-w-0 max-w-[280px] flex-[1_1_210px]">
      <div className="mb-[7px] flex items-baseline justify-between">
        <span className="text-label font-medium text-ink-muted">Profil complété</span>
        <span className="tabular text-item font-strong text-accent">{borne} %</span>
      </div>
      <div
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={borne}
        aria-label="Profil complété"
        className="mb-[9px] h-[7px] overflow-hidden rounded-tag bg-neutral-tint"
      >
        <div className="h-full rounded-tag bg-accent" style={{ width: `${borne}%` }} />
      </div>
      <p className="text-meta leading-normal text-ink-faint">{hint}</p>
    </div>
  );
}

export function ProfileSkeleton() {
  return (
    <div>
      <div className="border-b border-line bg-surface px-7 pt-[22px] pb-0">
        <div className="flex items-start gap-4">
          <Skeleton className="size-14 rounded-full" />
          <div className="flex-1 space-y-2">
            <Skeleton className="h-5 w-52" />
            <Skeleton className="h-3 w-72" />
          </div>
          <Skeleton className="h-16 w-56" />
        </div>
        <Skeleton className="mt-[18px] h-10 w-full" />
      </div>
    </div>
  );
}
