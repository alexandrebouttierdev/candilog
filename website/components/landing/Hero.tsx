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
 */
export function Hero() {
  return (
    <section aria-labelledby="hero-titre" className="overflow-x-clip">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col gap-7 px-4 pt-12 md:gap-9 md:px-8 md:pt-20 xl:px-0 xl:pt-24">
        <h1
          id="hero-titre"
          className="serif-title max-w-[1000px] text-balance text-[40px] leading-[1.04] tracking-[-0.03em] sm:text-[52px] lg:text-[66px] lg:leading-[1.02]"
        >
          Suivez chaque candidature.
          <br className="hidden sm:block" /> <span className="text-tx-4">Ciblez chaque document.</span>
        </h1>
        <div className="grid gap-7 lg:grid-cols-[560px_minmax(0,1fr)] lg:items-end lg:gap-20">
          <p className="text-pretty text-[16.5px] leading-[1.6] text-tx-3 md:text-[17.5px]">
            Candilog est une application de bureau pour mener votre recherche d&apos;emploi&nbsp;:
            candidatures, relances, entretiens, CV et lettres au même endroit. L&apos;IA, locale ou
            chez le fournisseur de votre choix, part des faits de votre profil pour adapter chaque
            document à l&apos;offre.
          </p>
          <div className="flex flex-col gap-[14px] lg:items-end">
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
        </div>
      </Reveal>

      <div className="mt-12 bg-[linear-gradient(var(--app)_0_120px,var(--desk)_120px)] md:mt-16 md:bg-[linear-gradient(var(--app)_0_200px,var(--desk)_200px)]">
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
