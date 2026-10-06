import { EcranAujourdhui } from "@/components/landing/app/ecrans/EcranAujourdhui";
import { ButtonLink } from "@/components/ui/Button";
import { DownloadMenu } from "@/components/ui/DownloadMenu";
import { Reveal } from "@/components/ui/Reveal";
import { GITHUB_REPO } from "@/lib/data/liens";

const FAITS = [
  {
    titre: "Sur votre ordinateur",
    texte: "Candidatures, documents et notes sont enregistrés localement. Aucun compte à créer.",
  },
  {
    titre: "L'IA que vous choisissez",
    texte: "IA locale installée depuis l'application, ou votre clé Mistral AI, OpenAI, Google Gemini, Anthropic, DeepSeek.",
  },
  {
    titre: "Des PDF d'une page",
    texte: "CV et lettres exportés en A4, texte sélectionnable. Rien n'est tronqué en silence.",
  },
  {
    titre: "macOS, Windows, Linux",
    texte: "Gratuit pour un usage personnel. Code source consultable sur GitHub.",
  },
] as const;

/**
 * Hero : la promesse en deux lignes, les deux actions (le téléchargement d'abord), puis
 * l'application elle-même — l'écran « Aujourd'hui » — posée sur le bureau (`desk`), comme
 * dans les captures de référence. Les quatre faits sous la fenêtre sont vérifiables dans l'application.
 *
 * Tout tient dans **une seule colonne de lecture**. Le titre, le chapô et les boutons se
 * lisaient auparavant en deux colonnes écartées de 80 px, l'appel à l'action collé au bord
 * droit : l'œil devait traverser un vide pour trouver le bouton qui suit la phrase qui le
 * justifie. La largeur, c'est la fenêtre d'application en dessous qui l'occupe.
 */
export function Hero() {
  return (
    <section aria-labelledby="hero-titre" className="overflow-x-clip">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col px-4 pt-12 md:px-8 md:pt-16 xl:px-0 xl:pt-20">
        <h1
          id="hero-titre"
          className="serif-title max-w-[860px] text-balance text-[34px] leading-[1.06] tracking-[-0.03em] sm:text-[44px] lg:text-[54px] lg:leading-[1.04]"
        >
          Suivez chaque candidature.
          {/* Coupure à toutes les largeurs : sans elle, sous 640 px, le changement d'encre
              tombait au milieu d'une ligne et les deux phrases n'en formaient plus qu'une.
              `tx-3` et non `tx-4` : à cette taille, le gris le plus clair du design passait
              pour une ligne désactivée au lieu d'un second temps. */}
          <br /> <span className="text-tx-3">Ciblez chaque document.</span>
        </h1>
        <p className="mt-5 max-w-[620px] text-pretty text-[16.5px] leading-[1.6] text-tx-3 md:text-[17.5px]">
          Candilog est une application de bureau pour mener votre recherche d&apos;emploi&nbsp;:
          candidatures, relances, entretiens, CV et lettres au même endroit. L&apos;IA, locale ou
          chez le fournisseur de votre choix, part des faits de votre profil pour adapter chaque
          document à l&apos;offre.
        </p>
        <div className="mt-7 flex flex-col gap-[14px]">
          <div className="flex flex-col gap-[10px] sm:flex-row">
            <DownloadMenu />
            <ButtonLink
              href={GITHUB_REPO}
              target="_blank"
              rel="noopener noreferrer"
              variante="secondaire"
              taille="tactile"
              className="sm:h-[40px] sm:px-4 sm:text-[14px]"
            >
              Voir le code source
            </ButtonLink>
          </div>
          <p className="font-mono text-[11.5px] text-tx-4">
            Gratuit pour un usage personnel · macOS · Windows · Linux · sans compte
          </p>
        </div>
      </Reveal>

      <div className="mt-10 bg-[linear-gradient(var(--app)_0_120px,var(--desk)_120px)] md:mt-14 md:bg-[linear-gradient(var(--app)_0_200px,var(--desk)_200px)]">
        <Reveal className="mx-auto max-w-[1200px] px-4 md:px-8 xl:px-0">
          <figure className="m-0">
            <figcaption className="sr-only">
              Suivi des candidatures dans Candilog, écran Aujourd&apos;hui&nbsp;: une relance en
              retard, un entretien à 14&nbsp;h&nbsp;30, trois échéances cette semaine et la
              répartition des 14 candidatures par statut.
            </figcaption>
            <div aria-hidden="true" className="window-lift">
              <EcranAujourdhui />
            </div>
          </figure>
          <ul
            aria-label="En bref"
            className="m-0 grid list-none grid-cols-2 gap-x-4 gap-y-7 px-0 pb-14 pt-10 md:gap-8 md:pb-[76px] md:pt-14 lg:grid-cols-4"
          >
            {FAITS.map(({ titre, texte }) => (
              <li key={titre} className="flex flex-col gap-[6px] border-l border-frame pl-3 md:pl-[18px]">
                <span className="text-[14px] font-semibold md:text-[14.5px]">{titre}</span>
                <span className="text-[13px] leading-[1.55] text-tx-3 md:text-[13.5px]">{texte}</span>
              </li>
            ))}
          </ul>
        </Reveal>
      </div>
    </section>
  );
}
