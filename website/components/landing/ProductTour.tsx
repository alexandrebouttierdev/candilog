"use client";

import { useId, useRef, useState, type KeyboardEvent } from "react";

import { Accessoire, BarreEtat, BarreTitre, Fenetre, Navigation, OngletsVue, Panneau } from "@/components/landing/app/Fenetre";
import { EcranAnalyse } from "@/components/landing/app/ecrans/EcranAnalyse";
import { EcranDocuments } from "@/components/landing/app/ecrans/EcranDocuments";
import { EcranListe } from "@/components/landing/app/ecrans/EcranListe";
import { EnTeteSection } from "@/components/landing/EnTeteSection";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";

const ONGLETS = [
  {
    cle: "candidatures",
    libelle: "Candidatures",
    aide: "Liste groupée par statut, filtres en tête de liste.",
    alt: "Liste des candidatures de Candilog groupée par statut, avec le filtre «\u00a0Statut n'est pas Refusée\u00a0» et la fiche de la candidature Product designer chez Studio Halage.",
    fil: ["Candidatures", "Toutes"] as const,
    nav: "applications" as const,
    etat: "11 candidatures sur 14 · filtre actif · 1 sélectionnée",
    touche: { libelle: "Statut", touche: "S" },
  },
  {
    cle: "documents",
    libelle: "Documents",
    aide: "CV et lettres, avec leur score et leurs versions.",
    alt: "Bibliothèque de documents de Candilog\u00a0: trois CV avec leur score ATS, trois lettres et le détail du CV Designer produit senior noté 84 sur 100 avec ses versions.",
    fil: ["Documents", "Tous"] as const,
    nav: "documents" as const,
    etat: "6 documents · 3 CV · 3 lettres · dernier export il y a 2 j",
    touche: { libelle: "Exporter", touche: "⌘E" },
  },
  {
    cle: "analyse",
    libelle: "Analyse",
    aide: "Des chiffres vérifiables, sans projection.",
    alt: "Analyse des candidatures dans Candilog\u00a0: 14 envoyées, 43\u00a0% de réponses, 3 entretiens, 9\u00a0jours de délai moyen, rythme d'envoi hebdomadaire et taux de réponse par canal.",
    fil: ["Candidatures", "Toutes"] as const,
    nav: "applications" as const,
    etat: "14 candidatures analysées · août → septembre 2026",
    touche: { libelle: "Exporter", touche: "⌘E" },
  },
] as const;

type Cle = (typeof ONGLETS)[number]["cle"];

const ECRAN: Record<Cle, () => React.ReactNode> = {
  candidatures: EcranListe,
  documents: EcranDocuments,
  analyse: EcranAnalyse,
};

/**
 * Section « Produit » : un grand aperçu de l'application, trois écrans au choix.
 *
 * Vrai `tablist` : `aria-selected`, tabindex itinérant, flèches gauche/droite,
 * Début/Fin. L'aperçu lui-même reste décoratif (`aria-hidden`) ; son contenu est décrit
 * par la légende du `tabpanel`.
 */
export function ProductTour() {
  const [actif, setActif] = useState<Cle>("candidatures");
  const id = useId();
  const boutons = useRef<Array<HTMLButtonElement | null>>([]);
  const onglet = ONGLETS.find((o) => o.cle === actif) ?? ONGLETS[0];
  const Ecran = ECRAN[onglet.cle];

  const auClavier = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = ONGLETS.findIndex((o) => o.cle === actif);
    const cible = {
      ArrowRight: (index + 1) % ONGLETS.length,
      ArrowLeft: (index - 1 + ONGLETS.length) % ONGLETS.length,
      Home: 0,
      End: ONGLETS.length - 1,
    }[event.key];
    if (cible === undefined) return;
    event.preventDefault();
    const suivant = ONGLETS[cible];
    if (!suivant) return;
    setActif(suivant.cle);
    boutons.current[cible]?.focus();
  };

  return (
    <section id="produit" aria-labelledby="produit-titre" className="border-t border-bd">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col gap-8 px-4 py-16 md:gap-10 md:px-8 md:py-24 xl:px-0 xl:py-32">
        <EnTeteSection
          id="produit-titre"
          surTitre="L'application"
          titre={
            <>
              Un écran par tâche.
              <br className="hidden sm:block" /> Rien qui ne serve pas.
            </>
          }
        >
          Pas de tableau de bord à widgets&nbsp;: une liste pour trier, une fiche pour décider, une
          barre d&apos;état qui dit ce qui se passe. Tout se pilote au clavier et{" "}
          <span className="font-mono text-[13px]">⌘K</span> ouvre toutes les actions.
        </EnTeteSection>

        <div className="flex flex-col gap-5">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div
              role="tablist"
              aria-label="Écrans de Candilog"
              onKeyDown={auClavier}
              className="flex gap-1 rounded-r10 bg-elev p-1"
            >
              {ONGLETS.map((o, i) => {
                const choisi = o.cle === actif;
                return (
                  <button
                    key={o.cle}
                    ref={(el) => {
                      boutons.current[i] = el;
                    }}
                    type="button"
                    role="tab"
                    id={`${id}-onglet-${o.cle}`}
                    aria-selected={choisi}
                    aria-controls={`${id}-panneau`}
                    tabIndex={choisi ? 0 : -1}
                    onClick={() => setActif(o.cle)}
                    className={cn(
                      "h-[40px] flex-1 rounded-r7 px-4 text-[14px] font-medium transition-colors duration-[160ms] sm:h-[34px] sm:flex-none sm:text-[13.5px]",
                      choisi ? "bg-panel text-tx shadow-[0_0_0_1px_var(--bd-menu)]" : "text-tx-3 hover:text-tx",
                    )}
                  >
                    {o.libelle}
                  </button>
                );
              })}
            </div>
            <p className="text-[13px] text-tx-4">{onglet.aide}</p>
          </div>

          <figure
            id={`${id}-panneau`}
            role="tabpanel"
            aria-labelledby={`${id}-onglet-${onglet.cle}`}
            className="m-0"
          >
            <figcaption className="sr-only">{onglet.alt}</figcaption>
            <div aria-hidden="true">
              <Fenetre hauteur="md:h-[620px]">
                <BarreTitre
                  fil={onglet.fil}
                  droite={
                    onglet.cle === "documents" ? (
                      <Accessoire>6 documents</Accessoire>
                    ) : (
                      <>
                        <OngletsVue actif={onglet.cle === "analyse" ? "Analyse" : "Liste"} />
                        <span className="hidden rounded-r6 bg-elev px-2 py-1 text-[11.5px] text-tx-3 lg:inline">
                          Grouper : statut ▾
                        </span>
                      </>
                    )
                  }
                />
                <div className="flex min-h-0 flex-1 overflow-hidden">
                  <Navigation active={onglet.nav} />
                  <Panneau key={onglet.cle} className="animate-pop">
                    <Ecran />
                  </Panneau>
                </div>
                <BarreEtat
                  gauche={onglet.etat}
                  touches={[
                    { libelle: "Ouvrir", touche: "⏎" },
                    onglet.touche,
                    { libelle: "Actions", touche: "⌘K", accent: true },
                  ]}
                />
              </Fenetre>
            </div>
          </figure>
        </div>
      </Reveal>
    </section>
  );
}
