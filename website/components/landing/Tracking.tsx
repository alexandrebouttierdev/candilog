"use client";

import { useEffect, useRef, useState } from "react";

import { BarreEtat, BarreTitre, Fenetre, Navigation, OngletsVue, Panneau } from "@/components/landing/app/Fenetre";
import { FicheCandidature } from "@/components/landing/app/ecrans/EcranListe";
import { Avatar, FauxBouton, Kbd, Pastille, StatusGlyph } from "@/components/landing/app/primitives";
import { EnTeteSection } from "@/components/landing/EnTeteSection";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { BANDEAUX, CANDIDATURES, STATUTS, candidaturesParStatut } from "@/lib/data/demo";

const BANDEAU = {
  c: "bg-tint-c-bg text-tint-c-tx",
  g: "bg-tint-g-bg text-tint-g-tx",
} as const;

/**
 * Section « Suivi » : le Kanban, grand, avec les quatorze candidatures du persona.
 *
 * Seul aperçu interactif du site : chaque carte est un vrai bouton qui ouvre la fiche
 * de la candidature, comme dans l'application (inspecteur flottant sous 1060 px).
 * `Échap` ou « ✕ » la referment et rendent le focus à la carte.
 */
export function Tracking() {
  const [choisie, setChoisie] = useState<string | null>(null);
  const cartes = useRef(new Map<string, HTMLButtonElement>());
  const fermer = useRef<HTMLButtonElement>(null);
  const fiche = CANDIDATURES.find((c) => c.ref === choisie);

  useEffect(() => {
    if (!choisie) return;
    fermer.current?.focus({ preventScroll: true });
    const surTouche = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      cartes.current.get(choisie)?.focus({ preventScroll: true });
      setChoisie(null);
    };
    document.addEventListener("keydown", surTouche);
    return () => document.removeEventListener("keydown", surTouche);
  }, [choisie]);

  const refermer = () => {
    if (choisie) cartes.current.get(choisie)?.focus({ preventScroll: true });
    setChoisie(null);
  };

  return (
    <section id="suivi" aria-labelledby="suivi-titre">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col gap-10 px-4 py-16 md:gap-12 md:px-8 md:py-24 xl:px-0 xl:py-32">
        <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_440px] lg:items-end lg:gap-20">
          <EnTeteSection
            id="suivi-titre"
            surTitre="Le suivi"
            empile
            titre={
              <>
                Quatre colonnes.
                <br className="hidden sm:block" /> Aucune candidature oubliée.
              </>
            }
          >
            Glissez une carte pour changer son statut. Les colonnes signalent d&apos;elles-mêmes ce
            qui attend depuis plus de 14&nbsp;jours et les entretiens du jour. Filtres, vues
            enregistrées et export CSV sont dans la même barre.
          </EnTeteSection>
          <ul aria-label="Les quatre statuts" className="m-0 flex list-none flex-col border-t border-bd p-0">
            {STATUTS.map(({ ton, libelle, sens }) => (
              <li key={ton} className="flex min-h-[44px] items-center gap-3 border-b border-bd py-2 text-[14px]">
                <StatusGlyph ton={ton} />
                <span className="w-[100px] flex-none font-medium">{libelle}</span>
                <span className="text-[13px] text-tx-4">{sens}</span>
              </li>
            ))}
          </ul>
        </div>

        <figure className="m-0 flex flex-col gap-[14px]">
          <figcaption className="sr-only">
            Kanban des candidatures de Candilog&nbsp;: quatorze candidatures réparties en quatre
            colonnes — En attente, Relancée, Entretien, Refusée. Chaque carte ouvre la fiche de
            la candidature.
          </figcaption>
          <Fenetre hauteur="md:h-[680px]" className="relative">
            <div aria-hidden="true">
              <BarreTitre fil={["Candidatures", "Toutes"]} droite={<OngletsVue actif="Kanban" />} />
            </div>
            <div className="flex min-h-0 flex-1 overflow-hidden">
              <div aria-hidden="true" className="contents">
                <Navigation active="applications" seuil="xl" />
              </div>
              <Panneau className="relative flex-col">
                <div
                  aria-hidden="true"
                  className="flex h-[38px] flex-none items-center gap-2 border-b border-bd-soft px-3 text-[12px]"
                >
                  <span className="text-tx-4">
                    + Filtre <Kbd>F</Kbd>
                  </span>
                  <span className="flex-1" />
                  <span className="hidden px-[6px] text-ac-tx sm:inline">Enregistrer la vue</span>
                  <FauxBouton visible="sm">CSV</FauxBouton>
                  <FauxBouton variante="primaire" touche="N">
                    Ajouter
                    <span className="hidden lg:inline">une candidature</span>
                  </FauxBouton>
                </div>
                <div className="no-scrollbar grid min-h-0 flex-1 snap-x snap-mandatory auto-cols-[minmax(248px,82%)] grid-flow-col gap-[10px] overflow-x-auto p-3 sm:auto-cols-[minmax(236px,1fr)] lg:grid-flow-row lg:grid-cols-4 lg:overflow-hidden">
                  {STATUTS.map(({ ton, libelle }) => {
                    const liste = candidaturesParStatut(ton);
                    const bandeau = BANDEAUX[ton];
                    return (
                      <div key={ton} className="flex min-w-0 snap-start flex-col gap-[7px] rounded-r9 bg-group p-2">
                        <h3 className="flex h-[26px] items-center gap-2 px-1 text-[12.5px] font-medium">
                          <StatusGlyph ton={ton} />
                          {libelle}
                          <span className="font-mono text-[10.5px] font-normal text-tx-5">{liste.length}</span>
                        </h3>
                        {bandeau ? (
                          <p className={cn("flex items-start gap-[7px] rounded-r7 px-[9px] py-[7px] text-[11px] leading-[1.35]", BANDEAU[bandeau.teinte])}>
                            <span aria-hidden="true" className="mt-1 size-[5px] flex-none rounded-full bg-current" />
                            {bandeau.texte}
                          </p>
                        ) : null}
                        {liste.map((c) => {
                          const active = c.ref === choisie;
                          return (
                            <button
                              key={c.ref}
                              ref={(el) => {
                                if (el) cartes.current.set(c.ref, el);
                                else cartes.current.delete(c.ref);
                              }}
                              type="button"
                              aria-pressed={active}
                              aria-label={`${c.poste}, ${c.entreprise}, ${c.ref}`}
                              onClick={() => setChoisie(active ? null : c.ref)}
                              className={cn(
                                "flex w-full flex-col gap-[6px] rounded-r8 border px-[10px] py-[9px] text-left transition-colors duration-[120ms]",
                                active ? "border-ac bg-sel" : "border-bd bg-panel hover:border-bd-menu hover:bg-hover",
                              )}
                            >
                              <span className="flex w-full items-center justify-between">
                                <span className="font-mono text-[10px] text-tx-5">{c.ref}</span>
                                {c.echeance ? (
                                  <Pastille teinte={c.echeance.teinte} mono petit>
                                    {c.echeance.texte}
                                  </Pastille>
                                ) : null}
                              </span>
                              <span className="w-full truncate text-[13px] font-medium text-tx">{c.poste}</span>
                              <span className="flex w-full items-center gap-[6px] text-[11.5px] text-tx-3">
                                <Avatar initiales={c.initiales} teinte={c.avatar} />
                                <span className="flex-1 truncate">{c.entreprise}</span>
                                <Pastille>{c.contrat}</Pastille>
                                <span className="font-mono text-[10px] text-tx-5">{c.envoyee}</span>
                              </span>
                            </button>
                          );
                        })}
                      </div>
                    );
                  })}
                </div>

                {fiche ? (
                  <aside
                    aria-label={`Fiche de la candidature ${fiche.ref}`}
                    className="absolute inset-y-2 right-2 flex w-[min(300px,calc(100%-16px))] animate-pop flex-col gap-[14px] overflow-y-auto rounded-r10 border border-bd-menu bg-modal px-[18px] py-4 shadow-pop"
                  >
                    <span className="flex items-center justify-between">
                      <span className="font-mono text-[10.5px] text-tx-5">{fiche.ref}</span>
                      <button
                        ref={fermer}
                        type="button"
                        onClick={refermer}
                        aria-label="Fermer la fiche"
                        className="grid size-[28px] place-items-center rounded-r6 bg-elev text-[11px] text-tx-3 hover:text-tx"
                      >
                        ✕
                      </button>
                    </span>
                    <FicheCandidature c={fiche} />
                  </aside>
                ) : null}
              </Panneau>
            </div>
            <div aria-hidden="true">
              <BarreEtat
                gauche="14 cartes · 4 colonnes · glissez pour changer de statut"
                touches={[
                  { libelle: "Statut", touche: "S" },
                  { libelle: "Relance", touche: "R" },
                  { libelle: "Actions", touche: "⌘K", accent: true },
                ]}
              />
            </div>
          </Fenetre>
          <p className="text-center text-[13px] text-tx-4">
            Touchez ou cliquez une carte pour ouvrir sa fiche.
            <span className="lg:hidden"> Faites défiler pour voir les quatre colonnes.</span>
          </p>
        </figure>
      </Reveal>
    </section>
  );
}
