import type { ReactNode } from "react";

import { Avatar, FauxBouton, FauxSegment, Pastille, StatusGlyph } from "@/components/landing/app/primitives";
import { EnTeteSection } from "@/components/landing/EnTeteSection";
import { Reveal } from "@/components/ui/Reveal";

/* Chaque étape montre un fragment réel de l'interface : la revue d'import du Profil,
   le formulaire de candidature, l'analyse face à l'offre, la feuille A4, le Kanban. */

function Cadre({ children, bureau = false }: { children: ReactNode; bureau?: boolean }) {
  return (
    <div
      aria-hidden="true"
      className={
        bureau
          ? "flex h-[196px] items-end justify-center gap-3 rounded-r10 border border-bd bg-desk p-3 text-[11px]"
          : "flex h-[196px] flex-col gap-2 rounded-r10 border border-bd bg-app p-3 text-[11px]"
      }
    >
      {children}
    </div>
  );
}

function Coche() {
  return (
    <span className="grid size-[10px] flex-none place-items-center rounded-r3 bg-ac text-[8px] leading-none text-on-accent">✓</span>
  );
}

const ETAPES: ReadonlyArray<{ titre: string; texte: string; ou: string; apercu: ReactNode }> = [
  {
    titre: "Importer son CV",
    texte: "L'IA lit votre CV et propose chaque élément. Vous validez un par un : rien n'est enregistré avant.",
    ou: "Profil › Importer un CV",
    apercu: (
      <Cadre>
        <span className="caps">Expériences · 3 proposées</span>
        <span className="flex items-center gap-[7px] rounded-r6 border border-bd bg-panel px-[7px] py-[6px]">
          <Coche />
          <span className="flex-1 truncate">Designer produit — Parcelle</span>
          <Pastille teinte="g">nouveau</Pastille>
        </span>
        <span className="flex items-center gap-[7px] rounded-r6 border border-bd bg-panel px-[7px] py-[6px]">
          <Coche />
          <span className="flex-1 truncate">Designer UX — Méridienne</span>
          <Pastille teinte="a">conflit</Pastille>
        </span>
        <span className="flex gap-1 pl-[17px]">
          <Pastille>Garder</Pastille>
          <Pastille teinte="ac">Remplacer</Pastille>
          <Pastille>Ajouter</Pastille>
        </span>
        <span className="flex-1" />
        <FauxBouton variante="primaire" touche="⌘S" className="self-start">
          Importer la sélection
        </FauxBouton>
      </Cadre>
    ),
  },
  {
    titre: "Ajouter une offre",
    texte: "Une candidature : poste, entreprise, contrat, lien de l'offre. Son texte se colle ensuite dans « Offre visée ».",
    ou: "Candidatures › Ajouter",
    apercu: (
      <Cadre>
        <span className="serif-title text-[13px]">Nouvelle candidature</span>
        <span className="flex flex-col gap-[3px]">
          <span className="text-[10.5px] text-tx-4">Poste</span>
          <span className="rounded-r6 border border-bd-menu bg-field px-[7px] py-[5px]">Designer produit senior</span>
        </span>
        <span className="flex flex-col gap-[3px]">
          <span className="text-[10.5px] text-tx-4">Entreprise</span>
          <span className="flex items-center gap-[6px] rounded-r6 border border-bd-menu bg-field px-[7px] py-1">
            <Avatar initiales="AN" teinte="av1" petit />
            Atelier Nord
          </span>
        </span>
        <FauxSegment options={["Réponse à une offre", "Spontanée"]} choisi="Réponse à une offre" />
      </Cadre>
    ),
  },
  {
    titre: "Analyser la correspondance",
    texte: "Chaque exigence de l'offre, couverte ou non, avec la preuve citée de votre CV — ou son absence.",
    ou: "Documents › Analyse face à l'offre",
    apercu: (
      <Cadre>
        <span className="serif-title text-[13px]">4 exigences couvertes sur 7</span>
        {(
          [
            ["g", "Construire un design system", "couverte", "g"],
            ["a", "Mesurer l'impact produit", "partielle", "ac"],
            ["c", "Accessibilité (RGAA)", "absente", "c"],
          ] as const
        ).map(([ton, texte, etat, teinte]) => (
          <span key={texte} className="flex items-center gap-[7px]">
            <StatusGlyph ton={ton} petit />
            <span className="flex-1 truncate">{texte}</span>
            <Pastille teinte={teinte}>{etat}</Pastille>
          </span>
        ))}
        <span className="flex-1" />
        <span className="flex items-baseline gap-[5px]">
          <span className="serif-title text-[20px] text-ac-tx">78</span>
          <span className="text-tx-4">/ 100 · jusqu&apos;à 91</span>
        </span>
      </Cadre>
    ),
  },
  {
    titre: "Générer les documents",
    texte: "CV ciblé et lettre, édités directement sur la page. Ce que vous voyez est ce qui sera exporté.",
    ou: "Documents › Générer un CV",
    apercu: (
      <Cadre bureau>
        <span className="flex h-[147px] w-[104px] flex-col gap-[3px] rounded-r2 bg-paper px-[9px] py-[11px] shadow-sheet">
          <span className="h-1 w-[62%] bg-paper-ink" />
          <span className="h-[2px] w-[44%] bg-paper-ink-3" />
          <span className="my-1 h-px bg-paper-rule" />
          <span className="h-[2px] bg-paper-sk" />
          <span className="h-[2px] bg-paper-sk" />
          <span className="h-[2px] w-[80%] bg-paper-sk" />
          <span className="mt-1 h-[2px] w-[40%] bg-paper-ink-2" />
          <span className="h-[2px] bg-paper-sk" />
          <span className="h-[2px] w-[90%] bg-paper-sk" />
          <span className="h-[2px] w-[70%] bg-paper-sk" />
          <span className="mt-1 h-[2px] w-[40%] bg-paper-ink-2" />
          <span className="h-[2px] w-[60%] bg-paper-sk" />
        </span>
        <span className="flex flex-col gap-[6px] pb-1">
          <FauxBouton variante="primaire" touche="⌘E">
            PDF
          </FauxBouton>
          <span className="font-mono text-[9.5px] text-tx-3">1 page A4</span>
        </span>
      </Cadre>
    ),
  },
  {
    titre: "Suivre la candidature",
    texte: "Relances et entretiens datés, retrouvés chaque matin dans Aujourd'hui et dans le calendrier.",
    ou: "Aujourd'hui · Calendrier",
    apercu: (
      <Cadre>
        <span className="flex items-center gap-[7px] font-medium">
          <StatusGlyph ton="a" petit />
          Relancée <span className="font-mono text-[10px] font-normal text-tx-5">4</span>
        </span>
        <span className="flex flex-col gap-[5px] rounded-r7 border border-ac bg-sel px-[9px] py-2">
          <span className="flex justify-between">
            <span className="font-mono text-[9.5px] text-tx-5">CAN-214</span>
            <Pastille teinte="ac" mono>
              ↻ 06-10
            </Pastille>
          </span>
          <span className="text-[12px] font-medium">Designer produit senior</span>
          <span className="flex items-center gap-[5px] text-tx-4">
            <Avatar initiales="AN" teinte="av1" petit />
            Atelier Nord
          </span>
        </span>
        <span className="flex-1" />
        <span className="self-start rounded-r6 bg-elev px-[7px] py-[5px] font-mono text-[10px] text-tx-3">
          CAN-214 → Relancée
        </span>
      </Cadre>
    ),
  },
];

/** Le parcours en cinq étapes, avec les vrais concepts de l'application. */
export function Workflow() {
  return (
    <section id="parcours" aria-labelledby="parcours-titre" className="border-y border-bd bg-panel">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col gap-10 px-4 py-16 md:gap-14 md:px-8 md:py-24 xl:px-0 xl:py-[120px]">
        <EnTeteSection
          id="parcours-titre"
          surTitre="Le parcours"
          titre={
            <>
              De l&apos;offre à l&apos;entretien,
              <br className="hidden sm:block" /> en cinq étapes.
            </>
          }
        >
          Votre profil est la source : chaque document en est une sélection. L&apos;IA choisit et
          reformule, elle n&apos;ajoute pas ce que vous n&apos;avez pas fait.
        </EnTeteSection>
        {/* Sous 1280 px, les étapes défilent horizontalement : une frise qu'on parcourt
            plutôt qu'une grille à trous. Dès 1280 px, les cinq tiennent sur une ligne. */}
        <ol className="no-scrollbar -mx-4 my-0 flex list-none snap-x snap-mandatory scroll-px-4 gap-5 overflow-x-auto px-4 pb-2 md:-mx-8 md:scroll-px-8 md:px-8 xl:mx-0 xl:grid xl:grid-cols-5 xl:overflow-visible xl:px-0">
          {ETAPES.map((etape, i) => (
            <li key={etape.titre} className="flex w-[78%] flex-none snap-start flex-col gap-[18px] sm:w-[260px] xl:w-auto">
              {etape.apercu}
              <div className="flex flex-col gap-2">
                <span className="font-mono text-[11.5px] text-ac-tx">{String(i + 1).padStart(2, "0")}</span>
                <h3 className="text-[16px] font-semibold">{etape.titre}</h3>
                <p className="text-pretty text-[13.5px] leading-[1.55] text-tx-3">{etape.texte}</p>
                <span className="font-mono text-[10.5px] text-tx-4">{etape.ou}</span>
              </div>
            </li>
          ))}
        </ol>
      </Reveal>
    </section>
  );
}
