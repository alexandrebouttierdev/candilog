import Image from "next/image";

import { EtiquetteMono } from "@/components/ui/EtiquetteMono";
import { Icon } from "@/components/ui/Icon";
import { cn } from "@/lib/cn";
import { STATUT, type CleStatut } from "@/lib/data/suivi";

/* Maquette de la fenêtre de l'application (§7.2). Purement décorative : aucun
   contrôle n'est un vrai bouton, rien n'est focusable, et l'ensemble est masqué
   aux lecteurs d'écran — le hero à gauche porte déjà tout le contenu utile. */

const RAIL = [
  "send",
  "domain",
  "group",
  "description",
  "calendar_month",
  "bar_chart",
];

const COLONNES = "grid-cols-[1.5fr_1fr_0.9fr_0.7fr]";

type Ligne = {
  initiales: string;
  poste: string;
  entreprise: string;
  statut: CleStatut;
  date: string;
  active?: boolean;
  attenuee?: boolean;
};

const LIGNES: Ligne[] = [
  {
    initiales: "AN",
    poste: "Designer produit",
    entreprise: "Atelier Nord",
    statut: "entretien",
    date: "02 août",
    active: true,
  },
  {
    initiales: "GV",
    poste: "Chargé de projet",
    entreprise: "Groupe Vallée",
    statut: "relance",
    date: "28 juil.",
  },
  {
    initiales: "MR",
    poste: "Assistant de direction",
    entreprise: "Maison Rivet",
    statut: "envoyee",
    date: "26 juil.",
  },
  {
    initiales: "CB",
    poste: "Coordinateur logistique",
    entreprise: "Cobalt Bureau",
    statut: "offre",
    date: "21 juil.",
  },
  {
    initiales: "LP",
    poste: "Gestionnaire de paie",
    entreprise: "Laurier & Pons",
    statut: "refus",
    date: "14 juil.",
    attenuee: true,
  },
];

const LIBELLE: Record<CleStatut, string> = {
  envoyee: "Envoyée",
  relance: "Relance",
  entretien: "Entretien",
  offre: "Offre reçue",
  refus: "Refus",
};

const FICHE = [
  { libelle: "Statut", valeur: "Entretien", accentuee: true },
  { libelle: "Entretien", valeur: "05 août · 14:30", mono: true },
  { libelle: "Relance", valeur: "08 août", mono: true },
  { libelle: "CV", valeur: "CV — Atelier Nord" },
  { libelle: "Contact", valeur: "Claire Nguyen" },
];

export function HeroAppWindow() {
  return (
    <div
      aria-hidden="true"
      /* La fenêtre déborde à droite au-delà de 1180px (§6) et perd son filet
         droit : elle sort du cadre plutôt que de s'y arrêter. */
      className="overflow-hidden rounded-l-panel border border-r-0 border-control bg-surface mr-[calc(-1*clamp(0px,(100vw-1180px)*0.35,110px))]"
    >
      {/* ── Barre de titre ──────────────────────────────────────────────── */}
      <div className="flex h-[46px] items-center gap-3 overflow-hidden border-b border-line-soft bg-surface-alt px-[14px]">
        <div className="flex gap-[6px]">
          {[0, 1, 2].map((point) => (
            <span key={point} className="size-[10px] rounded-full bg-line" />
          ))}
        </div>
        <span className="ml-[6px] text-[14.5px] font-semibold text-ink">
          Candidatures
        </span>
        <div className="ml-auto flex h-[26px] shrink-0 items-center gap-2 whitespace-nowrap rounded-control border border-line bg-surface px-[9px] text-[11.5px] text-ink-faint">
          <Icon name="search" size={14} />
          Rechercher ou exécuter…
          <span className="rounded-[4px] border border-line px-1 py-px font-mono text-[10.5px] text-ink-tertiary">
            ⌘K
          </span>
        </div>
      </div>

      {/* min-w-[540px] sur la zone de contenu : sous ~830px la fenêtre défile
          horizontalement plutôt que de s'écraser (§6). */}
      <div className="flex min-h-[472px] overflow-x-auto">
        {/* ── Rail latéral ──────────────────────────────────────────────── */}
        <div className="flex w-[68px] flex-[0_0_68px] flex-col items-center gap-1 border-r border-line-soft bg-surface-alt py-3">
          <div className="mb-[10px] grid size-[34px] place-items-center rounded-[9px] border border-line bg-surface">
            <Image
              src="/logo-candilog.svg"
              alt=""
              width={23}
              height={23}
              className="block"
            />
          </div>
          {RAIL.map((icone, index) => (
            <div
              key={icone}
              className={cn(
                "grid h-[34px] w-[38px] place-items-center rounded-control",
                index === 0 ? "bg-tint-12 text-accent-text" : "text-ink-faint",
              )}
            >
              <Icon name={icone} size={20} />
            </div>
          ))}
        </div>

        <div className="flex min-w-[540px] flex-1 flex-col">
          <div className="px-[18px] pb-3 pt-4">
            <p className="text-[13.5px] font-semibold text-ink">Candidatures</p>
            <p className="mt-1 text-[12px] text-ink-tertiary">
              Suivez chaque échange, de l&apos;envoi à la réponse.
            </p>
          </div>

          {/* ── Barre d'outils ──────────────────────────────────────────── */}
          <div className="flex flex-wrap items-center gap-2 border-b border-line-soft px-[18px] pb-3">
            <div className="flex h-[30px] w-[210px] items-center gap-[7px] rounded-control border border-control bg-surface px-[9px] text-[12px] text-ink-faint">
              <Icon name="search" size={15} />
              Rechercher…
            </div>
            <div className="flex h-[30px] items-center gap-[7px] rounded-control border border-control bg-surface px-[10px] text-[12.5px] font-semibold text-ink">
              <span className="text-ink-tertiary">
                <Icon name="tune" size={15} />
              </span>
              Filtres
              <span className="grid h-[15px] min-w-[15px] place-items-center rounded-[5px] bg-tint-12 px-1 text-[10.5px] text-accent-text">
                1
              </span>
            </div>
            <div className="inline-flex h-6 items-center gap-[6px] rounded-[7px] border border-tint-border bg-tint-08 px-2 text-[11.5px] font-semibold text-accent-text">
              Statut · Entretien
              <Icon name="close" size={13} />
            </div>
            <span className="ml-auto text-[12px] text-ink-tertiary">
              12 candidatures
            </span>
            <div className="inline-flex h-[30px] shrink-0 items-center gap-[6px] whitespace-nowrap rounded-control border border-accent-strong bg-accent px-[11px] text-[12.5px] font-semibold text-on-accent">
              <Icon name="add" size={15} />
              Nouvelle
            </div>
          </div>

          <div className="flex min-w-0 flex-1">
            {/* ── Table ─────────────────────────────────────────────────── */}
            <div className="min-w-0 flex-1">
              <div
                className={cn(
                  "grid gap-3 border-b border-line-soft bg-surface-alt px-[18px] py-[9px] font-mono text-[10.5px] font-semibold uppercase tracking-[0.07em] text-ink-faint",
                  COLONNES,
                )}
              >
                <span>Poste</span>
                <span>Entreprise</span>
                <span>Statut</span>
                <span className="text-right">Envoyée</span>
              </div>

              {LIGNES.map((ligne) => (
                <div
                  key={ligne.initiales}
                  className={cn(
                    "grid items-center gap-3 border-b border-line-soft px-[18px] py-[11px]",
                    COLONNES,
                    ligne.active && "bg-tint-06",
                    ligne.attenuee && "opacity-70",
                  )}
                >
                  <div className="flex min-w-0 items-center gap-[9px]">
                    <span
                      className={cn(
                        "grid size-[22px] shrink-0 place-items-center rounded-[7px] text-[10.5px] font-semibold",
                        ligne.active
                          ? "bg-tint-12 text-accent-text"
                          : "bg-page text-ink-muted",
                      )}
                    >
                      {ligne.initiales}
                    </span>
                    <span className="truncate text-[13px] font-semibold text-ink">
                      {ligne.poste}
                    </span>
                  </div>
                  <span className="text-[12.5px] text-ink-muted">
                    {ligne.entreprise}
                  </span>
                  <span>
                    <span
                      className={cn(
                        "inline-flex h-[19px] shrink-0 items-center whitespace-nowrap rounded-pill border px-[7px] text-[11px] font-semibold",
                        STATUT[ligne.statut],
                      )}
                    >
                      {LIBELLE[ligne.statut]}
                    </span>
                  </span>
                  <span className="text-right text-[12px] tabular-nums text-ink-tertiary">
                    {ligne.date}
                  </span>
                </div>
              ))}

              <div className="flex items-center gap-[10px] border-b border-line-soft bg-surface-alt px-[18px] py-[9px]">
                <span className="text-[11.5px] text-ink-faint">1–5 sur 12</span>
                <div className="ml-auto flex gap-1">
                  <span className="grid size-6 place-items-center rounded-[7px] border border-line bg-surface text-ink-faint">
                    <Icon name="chevron_left" size={14} />
                  </span>
                  <span className="grid size-6 place-items-center rounded-[7px] border border-line bg-surface text-ink-muted">
                    <Icon name="chevron_right" size={14} />
                  </span>
                </div>
              </div>
            </div>

            {/* ── Inspecteur ────────────────────────────────────────────── */}
            <div className="w-[246px] flex-[0_0_246px] border-l border-line-soft bg-surface-alt p-[14px]">
              <EtiquetteMono className="mb-[10px]">Fiche</EtiquetteMono>
              <p className="text-[14.5px] font-semibold leading-[1.25] text-ink">
                Designer produit
              </p>
              <p className="mt-[2px] text-[12px] text-ink-tertiary">
                Atelier Nord · Lyon
              </p>

              <div className="mt-[14px] border-t border-line-soft">
                {FICHE.map((ligne) => (
                  <div
                    key={ligne.libelle}
                    className="flex justify-between gap-[10px] border-b border-line-soft py-2"
                  >
                    <span className="text-[12px] text-ink-tertiary">
                      {ligne.libelle}
                    </span>
                    <span
                      className={cn(
                        "text-[12px] font-semibold",
                        ligne.accentuee ? "text-accent-text" : "text-ink",
                        ligne.mono && "tabular-nums",
                      )}
                    >
                      {ligne.valeur}
                    </span>
                  </div>
                ))}
              </div>

              <EtiquetteMono className="mb-2 mt-[14px]">Notes</EtiquetteMono>
              <p className="text-[12px] leading-[1.55] text-ink-muted">
                Demander le détail de l&apos;équipe design et le rythme de
                télétravail.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
