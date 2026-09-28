import type { ReactNode } from "react";

import { AnalyseOffre, GenerateurCv, GenerateurLettre } from "@/components/landing/app/ecrans/Generateurs";
import { StatusGlyph } from "@/components/landing/app/primitives";
import { EnTeteSection } from "@/components/landing/EnTeteSection";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import type { Ton } from "@/lib/data/demo";

type Fonction = {
  ecran: string;
  titre: string;
  texte: string;
  points: ReadonlyArray<{ ton: Ton; texte: string }>;
  apercu: ReactNode;
  legende: string;
};

/* Les trois surcouches de Documents, chacune avec ce qu'elle garantit réellement
   (`docs/CLAUDE_DESIGN_CONTEXT.md` §8, garanties IA). */
const FONCTIONS: readonly Fonction[] = [
  {
    ecran: "Générer un CV",
    titre: "Le CV qui répond à cette offre-là.",
    texte:
      "Choisissez la candidature, décidez ce que l'IA peut utiliser, puis générez. La page s'édite directement\u00a0; les propositions s'ajoutent ou s'ignorent une par une.",
    points: [
      { ton: "g", texte: "Expériences et formations toujours présentes\u00a0; seuls l'ordre et la mise en avant changent." },
      { ton: "a", texte: "Les compétences demandées mais absentes de votre profil restent signalées, jamais ajoutées." },
      { ton: "n", texte: "Une seule page A4. Si le contenu déborde, l'export est bloqué et la cause est nommée." },
    ],
    apercu: <GenerateurCv />,
    legende:
      "Générateur de CV de Candilog\u00a0: la candidature Designer produit senior chez Atelier Nord est choisie, le CV de Camille Berthier s'affiche sur une page A4, avec les étapes terminées en 16,2\u00a0secondes, un score de 84 sur 100 et deux propositions d'ajout.",
  },
  {
    ecran: "Analyser un CV",
    titre: "Ce que l'offre demande, et ce que votre CV prouve.",
    texte:
      "Déposez un CV en PDF, collez l'offre. Chaque exigence est classée couverte, partielle ou absente, avec l'extrait du CV qui le justifie.",
    points: [
      { ton: "g", texte: "Le score est calculé par Candilog, de façon déterministe — pas le chiffre renvoyé par le modèle." },
      { ton: "a", texte: "Une offre ou un PDF importé est traité comme une donnée, jamais comme des instructions." },
      { ton: "n", texte: "Une analyse est une indication, pas une garantie de sélection." },
    ],
    apercu: <AnalyseOffre />,
    legende:
      "Analyse face à l'offre dans Candilog\u00a0: 4 exigences couvertes sur 7, un score de 78 sur 100 et pour chaque exigence la preuve citée du CV ou la mention «\u00a0introuvable\u00a0».",
  },
  {
    ecran: "Générer une lettre",
    titre: "Une lettre assemblée à partir de votre parcours.",
    texte:
      "Le modèle choisit parmi les faits de votre profil\u00a0; il n'en invente pas. Candilog mesure ensuite ce qui répond vraiment à l'offre et propose quoi aborder.",
    points: [
      { ton: "g", texte: "Des recommandations appuyées sur un fait cité de votre profil, avec leur gain estimé." },
      { ton: "a", texte: "Des consignes qui se cumulent\u00a0: «\u00a0plus court\u00a0», puis «\u00a0moins formel\u00a0»." },
      { ton: "n", texte: "Ton, longueur et arguments autorisés restent à vous." },
    ],
    apercu: <GenerateurLettre />,
    legende:
      "Générateur de lettre de Candilog\u00a0: une lettre adressée à Atelier Nord sur une page A4, un score d'adéquation de 68 sur 100 pouvant atteindre 84 et des recommandations à appliquer ou ignorer.",
  },
];

/** Section « Documents » : trois rangées éditoriales, texte et aperçu en alternance. */
export function DocumentsShowcase() {
  return (
    <section id="documents" aria-labelledby="documents-titre" className="border-t border-bd">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col gap-16 px-4 py-16 md:gap-24 md:px-8 md:py-24 xl:gap-28 xl:px-0 xl:py-32">
        <EnTeteSection
          id="documents-titre"
          surTitre="CV, lettres, analyses"
          titre={
            <>
              Un document par offre,
              <br className="hidden sm:block" /> à partir de faits vérifiés.
            </>
          }
        >
          Votre profil réunit tout ce que vous avez fait. Pour chaque offre, Candilog en retient ce
          qui compte, sur une page A4 identique au PDF exporté.
        </EnTeteSection>

        {FONCTIONS.map((f, i) => (
          <article
            key={f.ecran}
            className="grid items-center gap-8 xl:grid-cols-[340px_minmax(0,1fr)] xl:gap-16"
          >
            <div className={cn("flex max-w-[640px] flex-col gap-5", i % 2 === 1 && "xl:order-2")}>
              <span className="font-mono text-[12px] text-ac-tx">{f.ecran}</span>
              <h3 className="serif-title text-balance text-[24px] leading-[1.15] md:text-[28px]">{f.titre}</h3>
              <p className="text-pretty text-[15px] leading-[1.65] text-tx-3">{f.texte}</p>
              <ul className="m-0 flex list-none flex-col border-t border-bd p-0">
                {f.points.map((p) => (
                  <li key={p.texte} className="flex gap-3 border-b border-bd py-[13px] text-[14px] leading-[1.5] text-tx-2">
                    <StatusGlyph ton={p.ton} className="mt-[5px]" />
                    <span>{p.texte}</span>
                  </li>
                ))}
              </ul>
            </div>
            <figure className="m-0 min-w-0">
              <figcaption className="sr-only">{f.legende}</figcaption>
              <div aria-hidden="true" className="window-lift">
                {f.apercu}
              </div>
            </figure>
          </article>
        ))}
      </Reveal>
    </section>
  );
}
