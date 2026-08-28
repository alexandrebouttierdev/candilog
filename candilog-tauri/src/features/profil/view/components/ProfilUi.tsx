import type { KeyboardEvent, ReactNode } from "react";
import type { Identite } from "@/shared/types/generated/profil";
import { cn } from "@/shared/lib/cn";
import { Button, Icon, Skeleton } from "@/shared/ui";

export type ProfilTab = "experiences" | "competences" | "formations" | "langues";

const TAB_LABELS: Record<ProfilTab, { label: string; icon: string }> = {
  experiences: { label: "Expériences", icon: "work_history" },
  competences: { label: "Compétences", icon: "psychology" },
  formations: { label: "Formations", icon: "school" },
  langues: { label: "Langues", icon: "translate" },
};

/** Navigation locale du profil, restituée comme de vrais onglets au clavier. */
export function ProfilTabs({
  active,
  counts,
  onChange,
}: {
  active: ProfilTab;
  counts: Record<ProfilTab, number>;
  onChange: (tab: ProfilTab) => void;
}) {
  return (
    <div role="tablist" aria-label="Sections du profil" className="flex gap-1 overflow-x-auto border-b border-line px-5">
      {(Object.keys(TAB_LABELS) as ProfilTab[]).map((tab) => {
        const meta = TAB_LABELS[tab];
        const tabs = Object.keys(TAB_LABELS) as ProfilTab[];
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
        return (
          <button
            key={tab}
            id={`profil-tab-${tab}`}
            type="button"
            role="tab"
            aria-selected={active === tab}
            aria-controls={`profil-panel-${tab}`}
            tabIndex={active === tab ? 0 : -1}
            onClick={() => onChange(tab)}
            onKeyDown={naviguer}
            className={cn(
              "relative flex h-12 flex-none items-center gap-2 px-3 text-body font-medium transition-colors duration-150",
              active === tab ? "text-accent" : "text-ink-muted hover:text-ink",
            )}
          >
            <Icon name={meta.icon} size={17} />
            {meta.label}
            <span className="tabular rounded-full bg-neutral-tint px-1.5 py-0.5 text-[10px] text-ink-faint">
              {counts[tab]}
            </span>
            {active === tab ? <span className="absolute inset-x-2 bottom-0 h-0.5 rounded-full bg-accent" /> : null}
          </button>
        );
      })}
    </div>
  );
}

export function ProfilPanel({
  tab,
  active,
  children,
}: {
  tab: ProfilTab;
  active: boolean;
  children: ReactNode;
}) {
  if (!active) return null;
  return (
    <div
      id={`profil-panel-${tab}`}
      role="tabpanel"
      aria-labelledby={`profil-tab-${tab}`}
      className="p-5"
    >
      {children}
    </div>
  );
}

export function SectionCard({
  icon,
  title,
  meta,
  onEdit,
  children,
}: {
  icon: string;
  title: string;
  meta?: string;
  onEdit: () => void;
  children: ReactNode;
}) {
  return (
    <section className="overflow-hidden rounded-card border border-line bg-surface shadow-e1">
      <header className="flex min-h-12 items-center gap-2 border-b border-line px-4">
        <Icon name={icon} size={17} className="text-accent" />
        <h2 className="min-w-0 flex-1 truncate text-section text-ink">{title}</h2>
        {meta ? <span className="text-meta text-ink-faint">{meta}</span> : null}
        <Button variant="ghost" icon="edit" onClick={onEdit} aria-label={`Modifier ${title.toLowerCase()}`}>
          Modifier
        </Button>
      </header>
      {children}
    </section>
  );
}

export function ProfileIdentity({ identite }: { identite: Identite }) {
  const nom = [identite.prenom, identite.nom].filter(Boolean).join(" ") || "Profil à compléter";
  const initiales = [identite.prenom, identite.nom]
    .filter(Boolean)
    .map((partie) => partie.charAt(0).toUpperCase())
    .join("") || "?";
  return (
    <div className="flex min-w-0 items-center gap-4">
      <span className="flex size-14 flex-none items-center justify-center rounded-[18px] bg-accent text-lg font-semibold text-white shadow-e1">
        {initiales}
      </span>
      <div className="min-w-0">
        <h2 className="truncate text-[22px] font-semibold tracking-[-0.02em] text-ink">{nom}</h2>
        <p className="truncate text-body text-ink-muted">{identite.titre ?? "Ajoutez votre objectif professionnel"}</p>
        <p className="mt-1 flex items-center gap-1.5 text-meta text-ink-faint">
          <Icon name="location_on" size={14} />
          {identite.ville ?? "Localisation non renseignée"}
        </p>
      </div>
    </div>
  );
}

export function CompletionRing({ value }: { value: number }) {
  const degres = Math.max(0, Math.min(100, value)) * 3.6;
  return (
    <div
      role="img"
      aria-label={`Profil complété à ${value} %`}
      className="relative flex size-20 flex-none items-center justify-center rounded-full"
      style={{ background: `conic-gradient(var(--color-accent) ${degres}deg, var(--color-neutral-tint) 0deg)` }}
    >
      <span className="absolute inset-[7px] rounded-full bg-surface" />
      <span className="tabular relative text-lg font-semibold text-ink">{value}%</span>
    </div>
  );
}

export function ProfilSkeleton() {
  return (
    <div className="space-y-4 p-5 min-[1200px]:p-6">
      <div className="rounded-card border border-line bg-surface p-6">
        <div className="flex items-center gap-4"><Skeleton className="size-14 rounded-[18px]" /><div className="flex-1 space-y-2"><Skeleton className="h-5 w-52" /><Skeleton className="h-3 w-72" /></div><Skeleton className="size-20 rounded-full" /></div>
      </div>
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_340px]">
        <Skeleton className="h-96 rounded-card" />
        <Skeleton className="h-72 rounded-card" />
      </div>
    </div>
  );
}
