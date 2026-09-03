import type { KeyboardEvent, ReactNode } from "react";
import type { Identity } from "@/shared/types/generated/profile";
import { cn } from "@/shared/lib/cn";
import { Button, Icon, IconButton, Skeleton } from "@/shared/ui";
import type { IconName } from "@/shared/ui/icon-names";

export type ProfileTab =
  | "experiences"
  | "skills"
  | "education"
  | "projects"
  | "certifications"
  | "languages";

const TAB_LABELS: Record<ProfileTab, { label: string; icon: IconName }> = {
  experiences: { label: "Expériences", icon: "work_history" },
  skills: { label: "Compétences", icon: "psychology" },
  education: { label: "Formations", icon: "school" },
  projects: { label: "Projets", icon: "rocket_launch" },
  certifications: { label: "Certifications", icon: "workspace_premium" },
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
  icon: IconName;
  title: string;
  actionLabel?: string;
  onEdit: () => void;
  children: ReactNode;
}) {
  return (
    <section className="overflow-hidden rounded-card border border-line bg-surface">
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

/**
 * Identité du bandeau : pastille, nom, coordonnées, et ce que l'appelant glisse dessous.
 *
 * La photo se change là où elle s'affiche : deux commandes de 24 px sous la pastille, au
 * lieu d'une carte séparée en bas de la colonne de droite. La suppression n'apparaît que
 * s'il y a quelque chose à supprimer.
 */
export function ProfileIdentity({
  identity,
  photo,
  busy,
  onPick,
  onRemove,
  children,
}: {
  identity: Identity;
  /** Photo en `data:` URL, ou `null` : la pastille retombe alors sur les initiales. */
  photo: string | null;
  busy: boolean;
  onPick: () => void;
  onRemove: () => void;
  /** Complément aligné sous les coordonnées, dans la colonne du nom. */
  children?: ReactNode;
}) {
  const name = [identity.first_name, identity.name].filter(Boolean).join(" ") || "Profil à compléter";
  const initials = [identity.first_name, identity.name]
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase())
    .join("") || "?";
  const chips = [
    identity.email ? { icon: "mail", label: identity.email } : null,
    identity.phone ? { icon: "call", label: identity.phone } : null,
    identity.city ? { icon: "location_on", label: identity.city } : null,
  ].filter((chip): chip is { icon: IconName; label: string } => chip !== null);

  return (
    <div className="flex min-w-[260px] flex-1 items-start gap-4">
      <div className="flex flex-none flex-col items-center gap-1.5">
        {photo ? (
          <img
            src={photo}
            alt="Photo de profil"
            className="size-16 rounded-full border border-line object-cover"
          />
        ) : (
          <span className="flex size-16 items-center justify-center rounded-full bg-accent-tint text-lg font-strong text-accent">
            {initials}
          </span>
        )}
        {/* Deux contrôles de 30 px : côte à côte, ils tiennent exactement la largeur de la
            pastille et gardent le gabarit standard du design system. */}
        <div className="flex gap-1">
          <IconButton
            icon={photo ? "swap_horiz" : "add_a_photo"}
            label={photo ? "Remplacer la photo" : "Ajouter une photo"}
            disabled={busy}
            onClick={onPick}
          />
          {photo ? (
            <IconButton
              icon="delete"
              label="Supprimer la photo"
              disabled={busy}
              onClick={onRemove}
            />
          ) : null}
        </div>
      </div>
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
        {children ? <div className="mt-3.5">{children}</div> : null}
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
          <Skeleton className="size-16 rounded-full" />
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

/** Action destructive du profil, isolée en bas de colonne et clairement signalée. */
export function ProfileResetCard({ busy, onReset }: { busy: boolean; onReset: () => void }) {
  return (
    <div className="rounded-card border border-danger-border bg-danger-tint px-[18px] py-4">
      <div className="mb-2 flex items-center gap-2">
        <Icon name="warning" size={18} className="text-danger" />
        <span className="text-item font-semibold text-danger">Réinitialiser le profil</span>
      </div>
      <p className="mb-3 text-label leading-[1.55] text-ink-muted">
        Efface uniquement les informations de votre profil. Vos candidatures et vos autres
        données restent intactes.
      </p>
      <Button variant="danger" icon="restart_alt" className="w-full" disabled={busy} onClick={onReset}>
        Réinitialiser mon profil
      </Button>
    </div>
  );
}
