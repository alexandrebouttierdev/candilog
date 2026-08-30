"use client";

import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { useAtsReveal } from "@/lib/hooks/useAtsReveal";

/* Toute la section vit sur les tokens --band-* : la bande a un contraste voulu,
   différent de celui de la page, dans les deux thèmes (§12).

   Exception assumée : le bouton « Adapter à l'offre ». Le prototype le peint
   `bg-accent` + `color: var(--band-ink-strong)`, soit du texte sombre sur indigo en
   thème clair — le seul bouton accent du site dans ce cas, à ~4,5:1. Arbitré avec
   l'auteur : il utilise `--on-accent` comme tous les autres. */

const BARRES = [
  {
    libelle: "Compétences couvertes",
    valeur: "9 / 14",
    ton: "accent" as const,
  },
  {
    libelle: "Vocabulaire de l'annonce",
    valeur: "Bon",
    ton: "accent" as const,
  },
  {
    libelle: "Lisibilité du document",
    valeur: "À vérifier",
    ton: "warning" as const,
  },
];

const POINTS = [
  { ok: false, texte: "« Accessibilité » n'apparaît pas dans le CV" },
  { ok: false, texte: "Deux colonnes : risque de mauvaise lecture" },
  { ok: true, texte: "Intitulé de poste aligné sur l'annonce" },
];

function EntetePanneau({
  icone,
  titre,
  meta,
  iconeAccentuee = false,
}: {
  icone: string;
  titre: string;
  meta: string;
  iconeAccentuee?: boolean;
}) {
  return (
    <div className="flex h-[38px] items-center gap-[9px] border-b border-band-line bg-band-alt px-3">
      <span
        className={iconeAccentuee ? "text-band-accent" : "text-band-ink-faint"}
      >
        <Icon name={icone} size={15} />
      </span>
      <span className="text-[12.5px] font-semibold text-band-ink">{titre}</span>
      <span className="ml-auto font-mono text-[10px] text-band-ink-faint">
        {meta}
      </span>
    </div>
  );
}

/** Terme de l'offre souligné en indigo. */
function Souligne({ children }: { children: React.ReactNode }) {
  return (
    <span className="border-b border-band-underline text-band-ink-body">
      {children}
    </span>
  );
}

export function AtsPanels() {
  const { ref, score, barres, anime } = useAtsReveal<HTMLDivElement>();

  return (
    <section
      id="cv"
      className="border-b border-band-line bg-band text-band-ink"
    >
      <Reveal className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)] py-[clamp(56px,7vw,104px)]">
        <div className="mb-9 flex flex-wrap items-end gap-6">
          <h2 className="max-w-[560px] text-[clamp(24px,2.8vw,36px)] font-semibold leading-[1.14] tracking-[-0.02em] text-band-ink-strong">
            Trois panneaux, une seule question :
            <br />
            ce CV répond-il à cette offre ?
          </h2>
          <p className="mb-[6px] max-w-[330px] text-[13.5px] leading-[1.65] text-band-ink-muted">
            L&apos;annonce à gauche, votre document au centre, l&apos;analyse à
            droite. Aucune promesse de résultat : des points précis à corriger.
          </p>
        </div>

        <div
          ref={ref}
          className="grid grid-cols-[repeat(auto-fit,minmax(min(300px,100%),1fr))] items-stretch gap-[14px]"
        >
          {/* ── Panneau 1 : l'offre ────────────────────────────────────── */}
          <div className="min-w-0 overflow-hidden rounded-card border border-band-line bg-band-surface">
            <EntetePanneau icone="work" titre="Offre" meta="Atelier Nord" />
            <div className="px-4 py-[14px] text-[12px] leading-[1.75] text-band-ink-muted">
              Nous cherchons une personne capable de mener une fonctionnalité de
              la <Souligne>recherche utilisateur</Souligne> à la mise en
              production, en binôme avec l&apos;équipe technique. Vous animerez
              les <Souligne>revues de design</Souligne> et ferez vivre notre{" "}
              <Souligne>système de composants</Souligne>. Une attention réelle à
              l&apos;<Souligne>accessibilité</Souligne> est attendue.
              <p className="mt-[14px] border-t border-band-line pt-3 font-mono text-[10px] uppercase tracking-[0.07em] text-band-ink-faint">
                14 éléments repérés
              </p>
            </div>
          </div>

          {/* ── Panneau 2 : le CV ──────────────────────────────────────── */}
          <div className="min-w-0 overflow-hidden rounded-card border border-band-line bg-band-surface">
            <EntetePanneau
              icone="description"
              titre="CV — Atelier Nord"
              meta="v3"
            />
            <div className="px-4 py-[14px]">
              <div className="rounded-tile border border-band-line bg-band-elevated p-[14px]">
                <p className="text-[12.5px] font-semibold text-band-ink-strong">
                  Camille Berthier
                </p>
                <p className="mt-[2px] text-[11px] text-band-ink-faint">
                  Designer produit · Lyon
                </p>
                <div className="my-[11px] h-px bg-band-line" />

                <p className="font-mono text-[9.5px] uppercase tracking-[0.07em] text-band-ink-faint">
                  Expérience
                </p>
                <p className="mt-[7px] text-[11px] leading-[1.7] text-band-ink-muted">
                  Refonte du parcours d&apos;inscription,{" "}
                  <span className="text-band-accent-soft">
                    entretiens utilisateurs
                  </span>{" "}
                  et animation des{" "}
                  <span className="text-band-accent-soft">
                    revues de design
                  </span>
                  .
                </p>

                <p className="mt-[11px] font-mono text-[9.5px] uppercase tracking-[0.07em] text-band-ink-faint">
                  Compétences
                </p>
                <p className="mt-[7px] text-[11px] leading-[1.7] text-band-ink-muted">
                  <span className="text-band-accent-soft">
                    Système de composants
                  </span>{" "}
                  · Prototypage · Ateliers
                </p>
              </div>

              <div className="mt-[10px] flex flex-wrap gap-[6px]">
                <span className="inline-flex h-[26px] shrink-0 items-center gap-[6px] whitespace-nowrap rounded-control border border-band-accent bg-accent px-[10px] text-[12px] font-semibold text-on-accent">
                  <Icon name="auto_fix_high" size={14} />
                  Adapter à l&apos;offre
                </span>
                <span className="inline-flex h-[26px] items-center rounded-control border border-band-control bg-band-elevated px-[10px] text-[12px] font-semibold text-band-ink-body">
                  Exporter
                </span>
              </div>
            </div>
          </div>

          {/* ── Panneau 3 : l'analyse ──────────────────────────────────── */}
          <div className="min-w-0 overflow-hidden rounded-card border border-band-line-accent bg-band-surface">
            <EntetePanneau
              icone="query_stats"
              titre="Analyse ATS"
              meta="02 août · 14:02"
              iconeAccentuee
            />
            <div className="p-4">
              <div className="flex items-baseline gap-2">
                <span className="text-[36px] font-semibold tabular-nums tracking-[-0.03em] text-band-ink-strong">
                  {score}
                </span>
                <span className="text-[13px] text-band-ink-faint">
                  / 100 de correspondance
                </span>
              </div>

              <div className="mt-[14px] flex flex-col gap-[11px]">
                {BARRES.map((barre, index) => (
                  <div key={barre.libelle}>
                    <div className="mb-[5px] flex justify-between text-[11.5px] text-band-ink-muted">
                      <span>{barre.libelle}</span>
                      <span className="tabular-nums">{barre.valeur}</span>
                    </div>
                    <div className="h-[5px] overflow-hidden rounded-[3px] bg-band-line">
                      <div
                        className={cn(
                          "h-full rounded-[3px]",
                          barre.ton === "accent"
                            ? "bg-band-accent"
                            : "bg-warning",
                          anime &&
                            "transition-[width] duration-[900ms] ease-out-soft",
                        )}
                        style={{ width: `${String(barres[index] ?? 0)}%` }}
                      />
                    </div>
                  </div>
                ))}
              </div>

              <p className="mt-4 font-mono text-[10px] uppercase tracking-[0.07em] text-band-ink-faint">
                Points à reprendre
              </p>
              <div className="mt-[9px] flex flex-col border-t border-band-line">
                {POINTS.map((point) => (
                  <div
                    key={point.texte}
                    className="flex gap-[9px] border-b border-band-line py-[9px]"
                  >
                    <span
                      className={point.ok ? "text-success" : "text-warning"}
                    >
                      <Icon
                        name={point.ok ? "check_circle" : "error"}
                        size={15}
                      />
                    </span>
                    <span className="text-[12px] text-band-ink-body">
                      {point.texte}
                    </span>
                  </div>
                ))}
              </div>

              <p className="mt-[14px] text-[11px] leading-[1.6] text-band-ink-faint">
                Une analyse est une indication, pas une garantie de sélection.
              </p>
            </div>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
