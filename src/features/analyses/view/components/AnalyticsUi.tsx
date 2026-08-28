import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import type {
  ARelancer,
  Echeance,
  Etape,
  Indicateurs,
  Performance,
  SemaineActivite,
} from "@/shared/types/generated/analyses";
import type { Candidature } from "@/shared/types/generated/candidatures";
import { contratLabel, statutMeta } from "@/features/candidatures/model/statuts";
import { cn } from "@/shared/lib/cn";
import {
  Button,
  CellIdentity,
  DataTable,
  EmptyState,
  Skeleton,
  StatusPill,
} from "@/shared/ui";
import type { Column, Tone } from "@/shared/ui";

/** Couleur de remplissage des barres et des pastilles, par tonalité. */
const FILL: Record<Tone, string> = {
  neutral: "bg-ink-faint",
  accent: "bg-accent",
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-danger",
};

const TINT: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Prochains entretiens et relances.
 *
 * Rangées de 11 px séparées par un filet haut, tuile de date de 36 px reprenant la teinte du
 * genre d'événement, pastille de genre à droite : la liste « Prochains événements » de la
 * maquette du tableau de bord.
 */
export function UpcomingList({ echeances }: { echeances: readonly Echeance[] }) {
  if (echeances.length === 0) {
    return (
      <EmptyState
        icon="event_available"
        title="Rien à venir"
        description="Aucun entretien ni relance n’est programmé."
      />
    );
  }

  return (
    <ul className="flex flex-col">
      {echeances.slice(0, 4).map((echeance) => {
        const entretien = echeance.genre === "entretien";
        const tone: Tone = entretien ? "success" : "warning";
        return (
          <li key={`${echeance.genre}-${echeance.id}`} className="border-t border-line">
            <Link
              to="/suivi/calendrier"
              className="flex items-center gap-3 py-[11px] transition-colors duration-150 hover:bg-neutral-tint"
            >
              <span
                className={cn(
                  "flex size-9 flex-none flex-col items-center justify-center rounded-tile leading-[1.05]",
                  TINT[tone],
                )}
              >
                <span className="tabular text-item font-strong">{jour(echeance.date)}</span>
                <span className="text-[8.5px] font-semibold tracking-[0.04em] uppercase">
                  {mois(echeance.date)}
                </span>
              </span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-item font-mid text-ink">
                  {entretien ? "Entretien" : "Relance"} — {echeance.entrepriseNom ?? "Entreprise"}
                </span>
                <span className="mt-0.5 block truncate text-label text-ink-faint">
                  {echeance.poste ?? "Candidature"} · {echeance.detail}
                </span>
              </span>
              <StatusPill tone={tone} icon={entretien ? "videocam" : "send"}>
                {entretien ? "Entretien" : "Relance"}
              </StatusPill>
            </Link>
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Histogramme des candidatures envoyées, semaine par semaine.
 *
 * Barres proportionnelles surmontées de leur valeur et suivies de leur libellé, comme dans
 * les maquettes. La hauteur du bloc de barres est un paramètre : le tableau de bord lui
 * donne 98 px, l'écran Analyses 150 px.
 *
 * Rendu en `div` plutôt qu'en SVG parce que les hauteurs sont exprimées en pourcentage du
 * conteneur : le graphique suit alors la largeur de la carte sans recalcul au
 * redimensionnement, ce qu'un `viewBox` fixe ne permettrait pas.
 */
export function ActivityChart({
  activite,
  height = 98,
  gap = 8,
}: {
  activite: readonly SemaineActivite[];
  height?: number;
  gap?: number;
}) {
  if (activite.every((semaine) => semaine.nombre === 0)) {
    return (
      <EmptyState
        icon="bar_chart"
        title="Pas encore d’activité"
        description="Les candidatures envoyées apparaîtront ici semaine après semaine."
      />
    );
  }

  const maximum = Math.max(...activite.map((semaine) => semaine.nombre), 1);

  return (
    <>
      <div
        role="img"
        aria-label="Candidatures envoyées par semaine"
        style={{ height, gap }}
        className="mb-[9px] flex items-end"
      >
        {activite.map((semaine) => (
          <div
            key={semaine.debut}
            className="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-1.5"
          >
            <span className="tabular text-eyebrow font-normal tracking-normal text-ink-faint">
              {semaine.nombre}
            </span>
            <div
              style={{ height: `${(semaine.nombre / maximum) * 100}%` }}
              className={cn(
                "min-h-1 w-full rounded-chip",
                semaine.nombre === 0 ? "bg-neutral-tint" : "bg-accent",
              )}
            />
          </div>
        ))}
      </div>
      <div style={{ gap }} className="flex" aria-hidden="true">
        {activite.map((semaine) => (
          <span
            key={semaine.debut}
            className="min-w-0 flex-1 truncate text-center text-eyebrow font-normal tracking-normal text-ink-faint"
          >
            {formatDate(semaine.debut, "court")}
          </span>
        ))}
      </div>
      <ol className="sr-only">
        {activite.map((semaine) => (
          <li key={semaine.debut}>
            Semaine du {formatDate(semaine.debut, "long")} : {semaine.nombre} candidature
            {semaine.nombre > 1 ? "s" : ""}
          </li>
        ))}
      </ol>
    </>
  );
}

/** Tonalités de l'entonnoir et du pipeline, dans l'ordre des étapes renvoyées par le backend. */
const ETAPES_PIPELINE: readonly Tone[] = ["neutral", "warning", "success", "danger"];
const ETAPES_ENTONNOIR: readonly Tone[] = ["accent", "accent", "success", "danger"];

/**
 * Entonnoir de conversion : une barre par étape, valeur et part à droite du libellé.
 */
export function FunnelChart({ etapes }: { etapes: readonly Etape[] }) {
  if (etapes.every((etape) => etape.nombre === 0)) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Entonnoir vide"
        description="Il se construira dès la première candidature."
      />
    );
  }

  return (
    <div>
      {etapes.map((etape, index) => (
        <div key={etape.label} className="mb-3.5 last:mb-0">
          <div className="mb-1.5 flex items-baseline justify-between gap-3">
            <span className="text-note font-medium text-ink-muted">{etape.label}</span>
            <span className="tabular text-note font-semibold text-ink">
              {etape.nombre}{" "}
              <span className="font-medium text-ink-faint">· {etape.pourcentage} %</span>
            </span>
          </div>
          <div className="h-2 overflow-hidden rounded-tag bg-neutral-tint">
            <div
              style={{ width: `${etape.pourcentage}%` }}
              className={cn("h-full rounded-tag", FILL[ETAPES_ENTONNOIR[index] ?? "neutral"])}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

/**
 * Répartition du pipeline : bande segmentée puis légende chiffrée.
 *
 * Les segments sont dimensionnés par `flex-grow` sur le nombre de candidatures, exactement
 * comme la maquette : une étape vide disparaît d'elle-même au lieu de laisser un filet.
 */
export function PipelineBar({ etapes }: { etapes: readonly Etape[] }) {
  const total = etapes.reduce((somme, etape) => somme + etape.nombre, 0);

  if (total === 0) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Pipeline vide"
        description="Les statuts de vos candidatures formeront cette répartition."
      />
    );
  }

  return (
    <div>
      <div className="mb-[13px] flex h-[9px] gap-[3px]">
        {etapes.map((etape, index) => (
          <span
            key={etape.label}
            style={{ flexGrow: etape.nombre, flexBasis: 0 }}
            className={cn("rounded-[3px]", FILL[ETAPES_PIPELINE[index] ?? "neutral"])}
          />
        ))}
      </div>
      <div className="flex flex-wrap gap-x-[22px] gap-y-2">
        {etapes.map((etape, index) => (
          <div key={etape.label} className="flex items-center gap-[7px]">
            <span
              className={cn(
                "size-1.5 rounded-full",
                FILL[ETAPES_PIPELINE[index] ?? "neutral"],
              )}
            />
            <span className="text-note text-ink-muted">{etape.label}</span>
            <span className="tabular text-note font-semibold text-ink">{etape.nombre}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

/** Colonnes du tableau des candidatures récentes, aux proportions de la maquette. */
function colonnesRecentes(): readonly Column<Candidature>[] {
  return [
    {
      key: "poste",
      header: "Poste",
      grow: 2.1,
      render: (candidature) => (
        <CellIdentity
          initials={initiales(candidature.entrepriseNom ?? candidature.poste)}
          title={candidature.poste}
          subtitle={`${contratLabel(candidature.typeContrat)}${
            candidature.entrepriseVille ? ` · ${candidature.entrepriseVille}` : ""
          }`}
        />
      ),
    },
    {
      key: "entreprise",
      header: "Entreprise",
      grow: 1.4,
      render: (candidature) => (
        <span className="truncate text-body text-ink-muted">
          {candidature.entrepriseNom ?? "—"}
        </span>
      ),
    },
    {
      key: "statut",
      header: "Statut",
      grow: 1.1,
      render: (candidature) => {
        const statut = statutMeta(candidature.statut);
        return (
          <StatusPill tone={statut.tone} icon={statut.icon}>
            {statut.label}
          </StatusPill>
        );
      },
    },
    {
      key: "maj",
      header: "Mise à jour",
      grow: 0.9,
      numeric: true,
      render: (candidature) => (
        <span className="text-note text-ink-faint">
          {formatDate(candidature.updatedAt, "long")}
        </span>
      ),
    },
  ];
}

/** Tableau des dernières candidatures, en pied du tableau de bord. */
export function RecentApplications({
  candidatures,
  header,
  onOuvrir,
}: {
  candidatures: readonly Candidature[];
  header: ReactNode;
  onOuvrir: (candidature: Candidature) => void;
}) {
  return (
    <DataTable
      columns={colonnesRecentes()}
      rows={candidatures}
      rowKey={(candidature) => candidature.id}
      onRowClick={onOuvrir}
      header={header}
      emptyState={
        <EmptyState
          icon="work"
          title="Aucune candidature"
          description="Ajoutez votre première candidature pour démarrer le suivi."
        />
      }
    />
  );
}

/**
 * Candidatures sans réponse, avec leur ancienneté et l'action de relance.
 *
 * L'ancienneté passe en rouge à quinze jours : au-delà, la maquette signale la relance comme
 * en retard plutôt que simplement due.
 */
export function FollowUpList({
  items,
  onRelancer,
}: {
  items: readonly ARelancer[];
  onRelancer: (item: ARelancer) => void;
}) {
  if (items.length === 0) {
    return (
      <EmptyState
        icon="task_alt"
        title="Aucune relance nécessaire"
        description="Toutes les candidatures récentes ont été traitées."
      />
    );
  }

  return (
    <ul>
      {items.map((item) => (
        <li
          key={item.id}
          className="flex items-center gap-[11px] border-b border-line px-[19px] py-3 last:border-b-0"
        >
          <span className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-meta font-strong text-ink-muted">
            {initiales(item.entrepriseNom ?? item.poste)}
          </span>
          <div className="min-w-0 flex-1">
            <p className="truncate text-body font-mid text-ink">{item.poste}</p>
            <p className="truncate text-label text-ink-faint">
              {item.entrepriseNom ?? "Entreprise"} · envoyée le {formatDate(item.dateEnvoi, "long")}
            </p>
          </div>
          <StatusPill tone={item.jours >= 15 ? "danger" : "warning"} compact>
            {item.jours} j
          </StatusPill>
          <Button icon="send" className="h-pager px-2.5 text-label" onClick={() => onRelancer(item)}>
            Relancer
          </Button>
        </li>
      ))}
    </ul>
  );
}

/** Rythme et délais : rangées libellé / valeur séparées par un filet haut. */
export function PerformanceList({
  performance,
  indicateurs,
}: {
  performance: Performance;
  indicateurs: Indicateurs;
}) {
  const lignes: readonly { label: string; valeur: string; tone?: Tone }[] = [
    {
      label: "Délai moyen de réponse",
      valeur:
        performance.delaiMoyenReponse === null ? "—" : `${performance.delaiMoyenReponse} j`,
    },
    {
      label: "Candidatures / semaine",
      valeur: formatNombre(performance.candidaturesParSemaine),
    },
    { label: "Taux d’entretien", valeur: `${indicateurs.tauxEntretien} %`, tone: "success" },
    {
      label: "Relances en retard",
      valeur: performance.relancesEnRetard.toString(),
      ...(performance.relancesEnRetard > 0 ? { tone: "warning" as Tone } : {}),
    },
  ];

  return (
    <dl>
      {lignes.map((ligne) => (
        <div
          key={ligne.label}
          className="flex items-center justify-between gap-3.5 border-t border-line py-2.5"
        >
          <dt className="min-w-0 text-note text-ink-muted">{ligne.label}</dt>
          <dd
            className={cn(
              "tabular flex-none text-body font-semibold",
              ligne.tone === "success"
                ? "text-success"
                : ligne.tone === "warning"
                  ? "text-warning"
                  : "text-ink",
            )}
          >
            {ligne.valeur}
          </dd>
        </div>
      ))}
    </dl>
  );
}

/** Ossature affichée pendant le chargement, aux dimensions des cartes réelles. */
export function AnalyticsSkeleton() {
  return (
    <div
      role="status"
      aria-label="Chargement en cours"
      className="px-7 pt-[22px] pb-8"
    >
      <div className="mb-4 grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(200px,100%),1fr))]">
        {Array.from({ length: 4 }, (_, index) => (
          <div key={index} className="rounded-card border border-line bg-surface px-[18px] py-4">
            <Skeleton className="h-3 w-24" />
            <Skeleton className="mt-[19px] h-6 w-16" />
          </div>
        ))}
      </div>
      <div className="mb-4 grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(320px,100%),1fr))]">
        <Skeleton className="h-[232px] rounded-card" />
        <Skeleton className="h-[232px] rounded-card" />
      </div>
      <Skeleton className="mb-4 h-[124px] rounded-card" />
      <Skeleton className="h-[300px] rounded-card" />
    </div>
  );
}

/** Initiales d'un intitulé, pour les pastilles des listes. */
export function initiales(valeur: string): string {
  return valeur
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}

function formatDate(valeur: string, format: "court" | "long"): string {
  const date = new Date(`${valeur.slice(0, 10)}T12:00:00`);
  if (Number.isNaN(date.getTime())) return valeur.slice(0, 10);
  return new Intl.DateTimeFormat("fr-FR", {
    day: format === "long" ? "2-digit" : "numeric",
    month: "short",
  }).format(date);
}

function jour(valeur: string): string {
  return valeur.slice(8, 10);
}

function mois(valeur: string): string {
  const date = new Date(`${valeur.slice(0, 10)}T12:00:00`);
  return Number.isNaN(date.getTime())
    ? ""
    : new Intl.DateTimeFormat("fr-FR", { month: "short" }).format(date).replace(".", "");
}

function formatNombre(valeur: number): string {
  return new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 }).format(valeur);
}
