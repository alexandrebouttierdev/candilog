import Image from "next/image";

import { Icon } from "@/components/ui/Icon";
import { cn } from "@/lib/cn";
import { COLONNES_BOARD, LIBELLE_STATUT, PUCE_STATUT } from "@/lib/data/suivi";

/* Maquette de la fenêtre de l'application (§7.2). Purement décorative : aucun
   contrôle n'est un vrai bouton, rien n'est focusable, et l'ensemble est masqué
   aux lecteurs d'écran — le hero à gauche porte déjà tout le contenu utile.

   La composition suit `AppShell` : rail de 68px, topbar de 46px à titre centré,
   sous-navigation de 186px, workspace, inspecteur de 380px. Candilog ne dessine pas
   sa propre barre de titre (`tauri.conf.json` garde les décorations natives) : cette
   maquette n'en invente donc aucune, ni pastilles macOS ni boutons Windows. */

/** Les sept sections de `Sections` (src/app/router/routes.tsx), dans l'ordre du rail. */
const RAIL = [
  { icone: "today", libelle: "Aujourd'hui" },
  { icone: "work", libelle: "Candidatures et calendrier" },
  { icone: "hub", libelle: "Entreprises et réseau" },
  { icone: "description", libelle: "CV et lettres de motivation" },
  { icone: "monitoring", libelle: "Statistiques" },
  { icone: "account_circle", libelle: "Profil professionnel" },
  { icone: "tune", libelle: "Intelligence artificielle et maintenance" },
];

/** Onglets de la section « Suivi », rendus par `SubNav` dès qu'elle a plus d'une route. */
const SOUS_NAV = [
  { icone: "work", libelle: "Candidatures", actif: true },
  { icone: "calendar_month", libelle: "Calendrier", actif: false },
];

export function HeroAppWindow() {
  return (
    <div
      aria-hidden="true"
      /* La fenêtre déborde à droite au-delà de 1180px (§6) et perd son filet
         droit : elle sort du cadre plutôt que de s'y arrêter. */
      className="overflow-hidden rounded-l-panel border border-r-0 border-control bg-surface mr-[calc(-1*clamp(0px,(100vw-1180px)*0.35,110px))]"
    >
      {/* La fenêtre garde les proportions réelles de l'application et se **coupe**
          au bord droit du cadre : une maquette décorative n'expose pas de barre de
          défilement. La vraie fenêtre fait 1440px par défaut, 1024 au minimum
          (`tauri.conf.json`) — bien plus que la place disponible ici. */}
      <div className="flex min-h-[520px] overflow-hidden">
        {/* ── Rail de navigation (68px) ──────────────────────────────────── */}
        <div className="flex w-[68px] flex-[0_0_68px] flex-col items-center border-r border-line-soft bg-surface-alt pt-3 pb-[10px]">
          <Image
            src="/logo-candilog.svg"
            alt=""
            width={36}
            height={36}
            className="mb-3 block size-9"
          />
          <span aria-hidden="true" className="mb-[10px] h-px w-6 bg-line" />

          <div className="flex flex-1 flex-col gap-1">
            {RAIL.map((section, index) => (
              <div
                key={section.icone}
                className={cn(
                  "grid h-9 w-[42px] place-items-center rounded-tile",
                  index === 1
                    ? "border border-tint-border bg-tint-10 text-accent-text"
                    : "text-ink-faint",
                )}
              >
                <Icon name={section.icone} size={20} />
              </div>
            ))}
          </div>

          {/* Dernier élément du rail : la bascule de thème, pas un réglage caché. */}
          <div className="mt-1 grid h-9 w-[42px] place-items-center rounded-tile text-ink-faint">
            <Icon name="dark_mode" size={20} />
          </div>
        </div>

        <div className="flex min-w-[760px] flex-1 flex-col">
          {/* ── Topbar (46px) : recherche à gauche, titre centré ───────────── */}
          <div className="grid h-[46px] flex-none grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2 border-b border-line-soft bg-surface-alt pr-3 pl-[14px]">
            <div className="col-start-1 row-start-1 justify-self-start">
              <div className="flex h-[29px] w-[220px] items-center gap-2 rounded-control border border-line bg-surface px-[10px] text-[11.5px] text-ink-faint">
                <Icon name="search" size={16} />
                <span className="min-w-0 flex-1 truncate text-left">
                  Rechercher ou exécuter…
                </span>
                <span className="rounded-[4px] border border-line px-1 py-px font-mono text-[10.5px] text-ink-tertiary">
                  ⌘K
                </span>
              </div>
            </div>

            <div className="col-start-2 row-start-1 flex items-center justify-center gap-2">
              <Icon name="work" size={17} className="text-ink-faint" />
              <span className="text-[13.5px] font-semibold text-ink">Candidatures</span>
            </div>
          </div>

          <div className="flex min-h-0 flex-1">
            {/* ── Sous-navigation (186px) ────────────────────────────────── */}
            <div className="flex w-[186px] flex-[0_0_186px] flex-col border-r border-line-soft bg-surface-alt py-3">
              <p className="mb-2 px-[18px] font-mono text-[10px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
                Suivi
              </p>
              <div className="flex flex-col gap-px px-[10px]">
                {SOUS_NAV.map((onglet) => (
                  <div
                    key={onglet.libelle}
                    className={cn(
                      "flex h-[30px] items-center gap-2 rounded-control px-2 text-[12.5px]",
                      onglet.actif
                        ? "bg-tint-12 font-semibold text-accent-text"
                        : "text-ink-tertiary",
                    )}
                  >
                    <Icon
                      name={onglet.icone}
                      size={16}
                      className={onglet.actif ? "text-accent-text" : "text-ink-faint"}
                    />
                    <span className="min-w-0 truncate">{onglet.libelle}</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex min-w-0 flex-1 flex-col">
              {/* ── Barre de filtres ────────────────────────────────────── */}
              <div className="flex min-h-[50px] flex-none flex-wrap items-center gap-2 border-b border-line-soft px-3 py-[10px]">
                <div className="flex h-[30px] w-[190px] items-center gap-[7px] rounded-control border border-control bg-surface px-[9px] text-[12px] text-ink-faint">
                  <Icon name="search" size={15} />
                  Rechercher…
                </div>
                <div className="inline-flex h-[30px] items-center gap-[6px] rounded-control border border-control bg-surface px-[11px] text-[12.5px] font-semibold text-ink">
                  <Icon name="filter_list" size={16} />
                  Filtres
                  <Icon name="expand_more" size={15} className="text-ink-faint" />
                </div>
                <span className="flex-1" />
                <span className="text-[12px] font-semibold tabular-nums text-ink">
                  14 candidatures
                </span>

                {/* Actions : bascule de vue, export, création — celles de la page. */}
                <div className="flex gap-[2px] rounded-[9px] border border-control bg-surface p-[2px]">
                  <span className="inline-flex h-[24px] items-center gap-[5px] rounded-[7px] bg-tint-12 px-[9px] text-[12px] font-semibold text-accent-text">
                    <Icon name="view_kanban" size={14} />
                    Kanban
                  </span>
                  <span className="inline-flex h-[24px] items-center gap-[5px] rounded-[7px] px-[9px] text-[12px] text-ink-muted">
                    <Icon name="view_list" size={14} />
                    Liste
                  </span>
                </div>
                <div className="inline-flex h-[30px] shrink-0 items-center gap-[6px] whitespace-nowrap rounded-control border border-control bg-surface px-[11px] text-[12.5px] font-semibold text-ink">
                  <Icon name="download" size={15} />
                  Exporter
                </div>
                <div className="inline-flex h-[30px] shrink-0 items-center gap-[6px] whitespace-nowrap rounded-control border border-accent-strong bg-accent px-[11px] text-[12.5px] font-semibold text-on-accent">
                  <Icon name="add" size={15} />
                  Nouvelle
                </div>
              </div>

              {/* ── Kanban : aucune ligne sélectionnée, donc pas d'inspecteur ── */}
              <div className="flex min-w-0 flex-1 gap-[14px] overflow-hidden p-[14px]">
                {COLONNES_BOARD.map((colonne) => (
                  <section
                    key={colonne.statut}
                    className="flex w-[240px] flex-none flex-col rounded-card border border-line bg-surface-alt"
                  >
                    <header className="flex flex-none items-center gap-2 border-b border-line px-[14px] py-3">
                      <span
                        className={cn(
                          "size-[7px] flex-none rounded-full",
                          PUCE_STATUT[colonne.statut],
                        )}
                      />
                      <h3 className="min-w-0 truncate text-[12.5px] font-semibold text-ink">
                        {LIBELLE_STATUT[colonne.statut]}
                      </h3>
                      <span className="flex-none rounded-[6px] border border-control bg-surface px-[6px] py-px text-[11px] font-semibold tabular-nums text-ink">
                        {colonne.cartes.length}
                      </span>
                      <span className="flex-1" />
                      <span className="grid size-[26px] place-items-center rounded-control text-ink-faint">
                        <Icon name="add" size={16} />
                      </span>
                    </header>

                    <div className="flex flex-1 flex-col gap-2 p-[10px]">
                      {colonne.cartes.slice(0, 3).map((carte) => (
                        <div
                          key={`${carte.entreprise}-${carte.poste}`}
                          className="min-w-0 rounded-tile border border-line bg-surface px-3 py-[10px]"
                        >
                          <div className="mb-[9px] flex items-start gap-[9px]">
                            <span className="grid size-[26px] flex-none place-items-center rounded-control bg-page text-[10.5px] font-semibold text-ink-muted">
                              {carte.initiales}
                            </span>
                            <div className="min-w-0 flex-1">
                              <p className="text-[12.5px] font-semibold leading-[1.35] text-ink">
                                {carte.poste}
                              </p>
                              <p className="mt-[2px] truncate text-[11.5px] text-ink-faint">
                                {carte.entreprise}
                              </p>
                            </div>
                          </div>
                          <div className="flex flex-wrap items-center gap-[6px]">
                            <span className="inline-flex h-[18px] items-center rounded-[6px] border border-line bg-surface-alt px-[6px] text-[10.5px] text-ink-muted">
                              {carte.contrat}
                            </span>
                            <span className="truncate text-[10.5px] text-ink-faint">
                              {carte.ville}
                            </span>
                            <span className="flex-1" />
                            <span
                              className={cn(
                                "inline-flex flex-none items-center gap-1 text-[10.5px] tabular-nums",
                                carte.jours >= 15 ? "text-warning" : "text-ink-faint",
                              )}
                            >
                              <Icon name={carte.jours >= 15 ? "schedule" : "event"} size={13} />
                              {carte.jours} j
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
