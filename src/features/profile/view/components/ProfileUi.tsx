import type { KeyboardEvent, ReactNode } from "react";
import type { Identity } from "@/shared/types/generated/profile";
import { cn } from "@/shared/lib/cn";
import { Button, Card, CardHeader, Icon, Skeleton } from "@/shared/ui";
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

export function ProfileIdentity({
  identity,
  photo,
}: {
  identity: Identity;
  /** Photo en `data:` URL, ou `null` : la pastille retombe alors sur les initiales. */
  photo?: string | null;
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
      {photo ? (
        // Décorative : le nom qu'elle accompagne est juste à côté, et la carte « Photo »
        // porte déjà l'aperçu nommé.
        <img
          src={photo}
          alt=""
          className="size-14 flex-none rounded-full border border-line object-cover"
        />
      ) : (
        <span className="flex size-14 flex-none items-center justify-center rounded-full bg-accent-tint text-lg font-strong text-accent">
          {initials}
        </span>
      )}
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

/**
 * Photo du profil : aperçu, remplacement, suppression.
 *
 * Facultative de bout en bout — le profil et les CV fonctionnent sans elle. Le cadre garde
 * le rapport de l'image (`object-contain`) : c'est celui que reprend l'export PDF.
 */
export function ProfilePhotoCard({
  photo,
  busy,
  onPick,
  onRemove,
}: {
  photo: string | null;
  busy: boolean;
  onPick: () => void;
  onRemove: () => void;
}) {
  return (
    <Card clipped>
      <CardHeader compact icon="photo_camera">
        Photo
      </CardHeader>
      <div className="px-[18px] pt-1 pb-4">
        {photo ? (
          <div className="flex items-start gap-3.5">
            <img
              src={photo}
              alt="Photo de profil"
              className="h-[92px] w-[78px] flex-none rounded-tile border border-line bg-surface-elevated object-contain"
            />
            <div className="flex min-w-0 flex-col gap-2">
              <p className="text-label leading-[1.55] text-ink-muted">
                Elle apparaît en haut à droite de vos CV, à son rapport d’origine.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button icon="swap_horiz" disabled={busy} onClick={onPick}>
                  Remplacer la photo
                </Button>
                <Button variant="ghost" icon="delete" disabled={busy} onClick={onRemove}>
                  Supprimer la photo
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex items-start gap-3.5">
            <span className="flex h-[92px] w-[78px] flex-none items-center justify-center rounded-tile border border-dashed border-line bg-surface-elevated text-ink-faint">
              <Icon name="photo_camera" size={22} />
            </span>
            <div className="flex min-w-0 flex-col gap-2">
              <p className="text-label leading-[1.55] text-ink-muted">
                Facultative. Sans photo, les CV se composent normalement, sans espace réservé.
              </p>
              <Button icon="add_a_photo" disabled={busy} onClick={onPick}>
                Ajouter une photo
              </Button>
            </div>
          </div>
        )}
      </div>
    </Card>
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
