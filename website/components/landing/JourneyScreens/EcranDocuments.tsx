import { Icon } from "@/components/ui/Icon";

import { EcranDeuxVolets, EtiquetteMono, Surlignage } from "./primitives";

const VERSIONS = [
  { icone: "description", libelle: "CV — Atelier Nord", date: "02 août", active: true },
  { icone: "description", libelle: "CV — base", date: "18 juil.", active: false },
  { icone: "mail", libelle: "Lettre — Atelier Nord", date: "02 août", active: false },
] as const;

/** Écran 03 — Documents : liste des versions et aperçu de CV avec trois surlignages. */
export function EcranDocuments() {
  return (
    <EcranDeuxVolets
      gauche={
        <>
          <EtiquetteMono>Versions du document</EtiquetteMono>
          <div className="mt-3 flex flex-col border-t border-line">
            {VERSIONS.map(({ icone, libelle, date, active }) => (
              <div
                key={libelle}
                className={`flex items-center gap-[10px] border-b border-line py-[9px] ${
                  active ? "bg-tint-06" : ""
                }`}
              >
                <span className={active ? "text-accent-text" : "text-ink-faint"}>
                  <Icon name={icone} size={16} />
                </span>
                <span className={`text-[12.5px] text-ink ${active ? "font-semibold" : ""}`}>
                  {libelle}
                </span>
                <span className="ml-auto text-[11.5px] tabular-nums text-ink-tertiary">{date}</span>
              </div>
            ))}
          </div>
          <p className="mt-4 text-[12px] leading-[1.65] text-ink-muted">
            Chaque version reste rattachée à sa candidature : vous savez toujours quel CV vous avez
            envoyé, et à qui.
          </p>
        </>
      }
      droite={
        <>
          <div className="rounded-tile border border-line bg-surface px-[18px] py-4">
            <p className="text-[13px] font-semibold text-ink">Camille Berthier</p>
            <p className="mt-[2px] text-[11.5px] text-ink-tertiary">Designer produit · Lyon</p>
            <div className="my-3 h-px bg-line-soft" />

            <p className="font-mono text-[10px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
              Expérience
            </p>
            <p className="mt-2 text-[11.5px] leading-[1.7] text-ink-muted">
              <Surlignage>Entretiens utilisateurs</Surlignage> et refonte du parcours
              d&apos;inscription, en binôme avec deux développeurs. Animation des{" "}
              <Surlignage>revues de design</Surlignage> hebdomadaires.
            </p>

            <p className="mt-3 font-mono text-[10px] font-semibold uppercase tracking-[0.07em] text-ink-faint">
              Compétences
            </p>
            <p className="mt-2 text-[11.5px] leading-[1.7] text-ink-muted">
              Système de composants · Prototypage · <Surlignage>Accessibilité</Surlignage>
            </p>
          </div>
          <p className="mt-[10px] font-mono text-[10.5px] text-ink-faint">
            3 ajouts issus de l&apos;analyse de l&apos;offre
          </p>
        </>
      }
    />
  );
}
