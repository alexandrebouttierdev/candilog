import { BrandIcon } from "@/components/ui/BrandIcon";
import { BrandMark } from "@/components/ui/BrandMark";
import { LineIcon } from "@/components/ui/LineIcon";
import { Reveal } from "@/components/ui/Reveal";
import { GITHUB_REPO } from "@/lib/data/liens";
import { PLATEFORMES } from "@/lib/data/plateformes";

/** Carré à quatre carreaux de Windows, dessiné : simple-icons n'a pas de glyphe utilisable. */
function LogoWindows() {
  return (
    <span aria-hidden="true" className="grid size-[15px] flex-none grid-cols-2 grid-rows-2 gap-[1.5px]">
      <span className="bg-current" />
      <span className="bg-current" />
      <span className="bg-current" />
      <span className="bg-current" />
    </span>
  );
}

/** Bloc de téléchargement : les quatre paquets, en liens directs vers la dernière
 *  version publiée (`lib/data/plateformes.ts`). */
export function DownloadCta() {
  return (
    <section id="telecharger" aria-labelledby="telecharger-titre">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col items-center gap-10 px-4 py-16 text-center md:px-8 md:py-24 xl:px-0 xl:py-32">
        <div className="flex flex-col items-center gap-[18px]">
          <BrandMark size={52} />
          <h2
            id="telecharger-titre"
            className="serif-title text-balance text-[34px] leading-[1.06] tracking-[-0.028em] md:text-[48px]"
          >
            Votre prochaine candidature
            <br className="hidden sm:block" /> commence ici.
          </h2>
          <p className="max-w-[480px] text-pretty text-[16px] leading-[1.6] text-tx-3">
            Installez Candilog, importez votre CV, ajoutez une première offre. Gratuit pour un
            usage personnel, sans compte.
          </p>
        </div>
        <ul aria-label="Téléchargements" className="m-0 grid w-full max-w-[1000px] list-none gap-3 p-0 text-left sm:grid-cols-2 xl:grid-cols-4">
          {PLATEFORMES.map((p) => (
            <li key={p.href}>
              <a
                href={p.href}
                className="group flex min-h-[64px] items-center gap-[14px] rounded-r10 border border-bd-menu bg-panel px-5 py-4 text-tx transition-colors duration-[120ms] hover:border-ac hover:bg-hover hover:text-tx"
              >
                <span className="text-tx-3">
                  {p.logo === "windows" ? <LogoWindows /> : <BrandIcon name={p.logo} size={15} />}
                </span>
                <span className="flex flex-1 flex-col gap-[3px]">
                  <span className="text-[15px] font-semibold">{p.groupe}</span>
                  <span className="text-[12.5px] text-tx-4">{p.libelle}</span>
                </span>
                <span className="font-mono text-[11px] text-tx-4">{p.extension}</span>
                <span className="text-ac-tx">
                  <LineIcon name="export-csv" size={14} strokeWidth={1.5} />
                </span>
              </a>
            </li>
          ))}
        </ul>
        <p className="font-mono text-[11.5px] text-tx-4">
          Toutes les versions et leurs notes sur{" "}
          <a
            href={`${GITHUB_REPO}/releases`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-ac-tx underline decoration-1 underline-offset-[3px]"
          >
            GitHub Releases
          </a>
        </p>
      </Reveal>
    </section>
  );
}
