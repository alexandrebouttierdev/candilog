import { Icon } from "@/components/ui/Icon";
import { cn } from "@/lib/cn";

import { EcranDeuxVolets, EtiquetteMono, Surlignage } from "./primitives";

/** Décisions ATS, dans les quatre états de `ResumeAtsPanel`. */
type EtatProposition = "decider" | "ajoutee" | "ignoree";

const PASTILLE: Record<EtatProposition, { classes: string; icone: string; libelle: string }> = {
  decider: {
    classes: "border-tint-border bg-tint-10 text-accent-text",
    icone: "query_stats",
    libelle: "À décider",
  },
  ajoutee: {
    classes: "border-success-border bg-success-tint text-success-text",
    icone: "check",
    libelle: "Ajoutée au CV",
  },
  ignoree: {
    classes: "border-line bg-page text-ink-muted",
    icone: "close",
    libelle: "Ignorée",
  },
};

const PROPOSITIONS = [
  {
    etat: "decider" as const,
    titre: "Ajouter « Recherche quantitative » aux compétences",
    texte: "Recherche quantitative",
    projete: 89,
  },
  {
    etat: "ajoutee" as const,
    titre: "Mentionner les entretiens utilisateurs de la refonte",
    texte:
      "Refonte du parcours d'inscription, de la recherche utilisateur à la mise en production",
  },
  {
    etat: "ignoree" as const,
    titre: "Ajouter « Design ops » aux compétences",
    texte: "Design ops",
  },
];

/** Écran 02 — les décisions ATS de l'éditeur de CV.
 *
 *  L'application ne rend ni jauge « lisibilité », ni verdict global : un score sur 100, le
 *  gain accumulé depuis la génération, puis une proposition par carte, acceptée ou ignorée
 *  une par une. Rien ne s'applique tout seul. */
export function EcranAnalyse() {
  return (
    <EcranDeuxVolets
      colonneMin={280}
      gauche={
        <>
        <div className="mb-3 flex items-center gap-[7px]">
          <span className="text-ink-faint">
            <Icon name="article" size={15} />
          </span>
          <span className="text-[12.5px] font-semibold text-ink">Aperçu HTML · A4</span>
        </div>

        <div className="rounded-tile border border-line bg-surface px-[18px] py-4">
          <p className="text-[13px] font-semibold text-ink">Camille Berthier</p>
          <p className="mt-[2px] text-[11.5px] text-ink-tertiary">Designer produit · Lyon</p>
          <div className="my-3 h-px bg-line-soft" />

          <EtiquetteMono className="text-[10px]">Profil</EtiquetteMono>
          <p className="mt-2 text-[11.5px] leading-[1.7] text-ink-muted">
            Designer produit, six ans d&apos;expérience sur des interfaces de gestion. J&apos;ai
            mené des <Surlignage>entretiens utilisateurs</Surlignage> sur la refonte du parcours
            d&apos;inscription, construit un système de composants documenté et animé les revues
            de design hebdomadaires.
          </p>

          <EtiquetteMono className="mt-3 text-[10px]">Compétences</EtiquetteMono>
          <p className="mt-2 text-[11.5px] leading-[1.7] text-ink-muted">
            Système de composants · Prototypage · <Surlignage>Accessibilité</Surlignage>
          </p>
        </div>
        </>
      }
      droite={
        <>
        <div className="mb-3 flex items-center gap-[7px]">
          <span className="text-ink-faint">
            <Icon name="query_stats" size={15} />
          </span>
          <span className="text-[12.5px] font-semibold text-ink">Analyse ATS</span>
        </div>

        {/* Score sur 100, cerclé — vert au-delà de 70, ambre au-delà de 45. */}
        <div className="flex items-center gap-3">
          <span className="grid size-12 place-items-center rounded-full border-4 border-success text-[12px] font-semibold tabular-nums text-success">
            84
          </span>
          <div>
            <p className="text-[12px] font-medium text-ink">Score ATS</p>
            <p className="text-[11px] text-ink-tertiary">sur 100</p>
          </div>
          <span className="ml-auto text-[12px] font-medium text-success">+13 points</span>
        </div>

        <div className="mt-4 flex flex-col gap-2">
          {PROPOSITIONS.map((proposition) => {
            const pastille = PASTILLE[proposition.etat];
            return (
              <div
                key={proposition.titre}
                className="rounded-card border border-line bg-surface p-[14px]"
              >
                <div className="flex items-start justify-between gap-2">
                  <p className="min-w-0 flex-1 text-[11.5px] font-semibold text-ink">
                    {proposition.titre}
                  </p>
                  <span
                    className={cn(
                      "inline-flex h-[19px] shrink-0 items-center gap-[4px] whitespace-nowrap rounded-pill border px-[7px] text-[10.5px] font-semibold",
                      pastille.classes,
                    )}
                  >
                    <Icon name={pastille.icone} size={12} />
                    {pastille.libelle}
                  </span>
                </div>

                <p className="mt-[8px] text-[11.5px] leading-[1.6] text-ink-muted">
                  {proposition.texte}
                </p>

                {proposition.etat === "decider" ? (
                  <>
                    <p className="mt-[8px] text-[11px] text-ink-tertiary">
                      Score projeté{" "}
                      <strong className="font-semibold text-ink">{proposition.projete}</strong>
                    </p>
                    <div className="mt-[10px] flex gap-[6px]">
                      <span className="inline-flex h-[26px] items-center rounded-control border border-accent-strong bg-accent px-[10px] text-[12px] font-semibold text-on-accent">
                        Accepter
                      </span>
                      <span className="inline-flex h-[26px] items-center rounded-control px-[10px] text-[12px] font-semibold text-ink-muted">
                        Refuser
                      </span>
                    </div>
                  </>
                ) : (
                  <div className="mt-[10px]">
                    <span className="inline-flex h-[26px] items-center rounded-control px-[10px] text-[12px] font-semibold text-ink-muted">
                      Annuler
                    </span>
                  </div>
                )}
              </div>
            );
          })}
        </div>
        </>
      }
    />
  );
}
