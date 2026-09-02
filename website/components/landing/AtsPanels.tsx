"use client";

import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { GAIN_CUMULE, useAtsReveal } from "@/lib/hooks/useAtsReveal";

/* Toute la section vit sur les tokens --band-* : la bande a un contraste voulu,
   différent de celui de la page, dans les deux thèmes (§12).

   Le panneau d'analyse reproduit `ResumeAtsPanel` : un score sur 100, le gain
   accumulé depuis la génération, puis une proposition par carte. L'application ne
   produit ni jauge de lisibilité, ni verdict global — une mesure qu'elle ne calcule
   pas n'a rien à faire ici. */

/** Les quatre états d'une proposition dans `ResumeAtsPanel`. */
const ETATS = {
  decider: { classes: "border-band-line-accent text-band-accent", libelle: "À décider", icone: "query_stats" },
  ajoutee: { classes: "border-success-border text-success", libelle: "Ajoutée au CV", icone: "check" },
  ignoree: { classes: "border-band-line text-band-ink-faint", libelle: "Ignorée", icone: "close" },
  inapplicable: { classes: "border-band-line text-band-ink-faint", libelle: "Non applicable", icone: "block" },
} as const;

const PROPOSITIONS = [
  {
    etat: "decider" as const,
    titre: "Ajouter « Recherche quantitative » aux compétences",
    detail: "Score projeté 89",
  },
  {
    etat: "ajoutee" as const,
    titre: "Mentionner les entretiens utilisateurs de la refonte",
    detail: "+6 points",
  },
  {
    etat: "ignoree" as const,
    titre: "Ajouter « Design ops » aux compétences",
    detail: "Décision annulable",
  },
  {
    etat: "inapplicable" as const,
    titre: "Reformuler l'accroche du profil",
    detail: "Ne s'applique plus au CV actuel",
  },
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
  const { ref, score } = useAtsReveal<HTMLDivElement>();

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
                12 éléments repérés
              </p>
            </div>
          </div>

          {/* ── Panneau 2 : le CV ──────────────────────────────────────── */}
          <div className="min-w-0 overflow-hidden rounded-card border border-band-line bg-band-surface">
            <EntetePanneau
              icone="description"
              titre="CV — Atelier Nord"
              meta="02-08-2026"
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
                  <Icon name="save" size={14} />
                  Enregistrer
                </span>
                <span className="inline-flex h-[26px] shrink-0 items-center gap-[6px] whitespace-nowrap rounded-control border border-band-control bg-band-elevated px-[10px] text-[12px] font-semibold text-band-ink-body">
                  <Icon name="picture_as_pdf" size={14} />
                  Exporter en PDF
                </span>
              </div>
            </div>
          </div>

          {/* ── Panneau 3 : l'analyse ──────────────────────────────────── */}
          <div className="min-w-0 overflow-hidden rounded-card border border-band-line-accent bg-band-surface">
            <EntetePanneau
              icone="query_stats"
              titre="Analyse ATS"
              meta="02-08-2026"
              iconeAccentuee
            />
            <div className="p-4">
              <div className="flex items-baseline gap-2">
                <span className="text-[36px] font-semibold tabular-nums tracking-[-0.03em] text-band-ink-strong">
                  {score}
                </span>
                <span className="text-[13px] text-band-ink-faint">/ 100</span>
                <span className="ml-auto text-[12.5px] font-semibold text-success">
                  +{GAIN_CUMULE} points
                </span>
              </div>
              <p className="mt-[3px] text-[11.5px] text-band-ink-faint">
                Score ATS, et gain depuis la génération
              </p>

              <p className="mt-4 font-mono text-[10px] uppercase tracking-[0.07em] text-band-ink-faint">
                Propositions
              </p>
              <div className="mt-[9px] flex flex-col gap-2">
                {PROPOSITIONS.map((proposition) => {
                  const etat = ETATS[proposition.etat];
                  return (
                    <div
                      key={proposition.titre}
                      className="rounded-tile border border-band-line bg-band-elevated p-[11px]"
                    >
                      <div className="flex items-start justify-between gap-2">
                        <p className="min-w-0 flex-1 text-[11.5px] font-semibold text-band-ink-body">
                          {proposition.titre}
                        </p>
                        <span
                          className={cn(
                            "inline-flex h-[19px] shrink-0 items-center gap-[4px] whitespace-nowrap rounded-pill border px-[6px] text-[10.5px] font-semibold",
                            etat.classes,
                          )}
                        >
                          <Icon name={etat.icone} size={11} />
                          {etat.libelle}
                        </span>
                      </div>
                      <p className="mt-[5px] text-[11px] text-band-ink-faint">
                        {proposition.detail}
                      </p>
                    </div>
                  );
                })}
              </div>

              <p className="mt-[14px] text-[11px] leading-[1.6] text-band-ink-faint">
                Aucune proposition ne s&apos;applique sans votre accord, et chaque
                décision s&apos;annule.
              </p>
            </div>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
