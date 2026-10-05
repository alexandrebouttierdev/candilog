import type { ReactNode } from "react";

import { BarreEtat, BarreTitre, Fenetre } from "@/components/landing/app/Fenetre";
import {
  Avatar,
  FauxBouton,
  FauxSegment,
  FauxSwitch,
  Pastille,
  StatusGlyph,
} from "@/components/landing/app/primitives";
import { cn } from "@/lib/cn";
import {
  DETAIL_SCORE,
  ETAPES_CV,
  ETAPES_LETTRE,
  EXIGENCES,
  PERSONA,
  SECTIONS_AUTORISEES,
} from "@/lib/data/demo";

/*
 * Surcouches plein écran de Documents (`WorkSurface`) : générateur de CV, analyse face
 * à l'offre, générateur de lettre (`screens/15`, `09`, `16`). Trois colonnes dès
 * 1024 px ; en dessous, la colonne de droite passe sous la feuille et celle de gauche
 * disparaît — l'aperçu garde la page A4 lisible plutôt que de la comprimer.
 */

const TEINTE_ETAT = { couverte: "g", partielle: "ac", absente: "c" } as const;
const BARRE = { ac: "bg-ac", g: "bg-st-g" } as const;

function Etapes({ etapes }: { etapes: ReadonlyArray<{ libelle: string; duree: string }> }) {
  return (
    <>
      <span className="caps">Terminé</span>
      {etapes.map(({ libelle, duree }) => (
        <span key={libelle} className="flex items-center gap-2">
          <span className="size-[7px] rounded-full bg-st-g" />
          <span className="flex-1">{libelle}</span>
          <span className="font-mono text-[10px] text-tx-5">{duree}</span>
        </span>
      ))}
    </>
  );
}

function Proposition({
  titre,
  gain,
  detail,
  actions,
  glyphe = false,
}: {
  titre: string;
  gain: string;
  detail: string;
  actions?: ReactNode;
  glyphe?: boolean;
}) {
  return (
    <span className="flex flex-col gap-[6px] rounded-r8 border border-bd bg-panel p-[9px]">
      <span className="flex justify-between gap-[6px]">
        <span className="flex gap-[6px]">
          {glyphe ? <StatusGlyph ton="a" className="mt-[2px]" /> : null}
          <span className="font-medium leading-[1.35]">{titre}</span>
        </span>
        <Pastille teinte="ac" mono>
          {gain}
        </Pastille>
      </span>
      <span className="text-[10.5px] leading-[1.4] text-tx-4">{detail}</span>
      {actions ? <span className="flex gap-[5px]">{actions}</span> : null}
    </span>
  );
}

function Colonne({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("flex flex-col gap-[10px] bg-app p-[14px] text-[11.5px]", className)}>{children}</div>;
}

function Bureau({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div className={cn("flex min-w-0 flex-1 justify-center overflow-hidden bg-desk pt-[22px]", className)}>
      {children}
    </div>
  );
}

/* Libellé de section de la feuille : IBM Plex Mono en capitales, comme le gabarit. */
function TitreFeuille({ children }: { children: ReactNode }) {
  return <span className="font-mono text-[6px] uppercase tracking-[0.08em] text-paper-ink-3">{children}</span>;
}

function LigneFeuille({ titre, children }: { titre: string; children: ReactNode }) {
  return (
    <span className="grid grid-cols-[58px_1fr] gap-2">
      <TitreFeuille>{titre}</TitreFeuille>
      {children}
    </span>
  );
}

function Poste({ intitule, periode, puces }: { intitule: string; periode: string; puces: readonly string[] }) {
  return (
    <span className="flex flex-col gap-[2px]">
      <span className="flex justify-between gap-2">
        <span className="font-semibold">{intitule}</span>
        <span className="font-mono text-[6px] text-paper-ink-3">{periode}</span>
      </span>
      {puces.map((p) => (
        <span key={p} className="text-paper-ink-2">
          · {p}
        </span>
      ))}
    </span>
  );
}

/** Générateur de CV : candidature visée, sections autorisées, feuille A4, étapes,
 *  score, propositions et compétences manquantes. */
export function GenerateurCv() {
  return (
    <Fenetre hauteur="md:h-[600px]">
      <BarreTitre
        surcouche
        fil={["Documents", "Générer un CV"]}
        droite={
          <span className="flex gap-[6px]">
            <FauxBouton visible="sm">Exporter en PDF</FauxBouton>
            <FauxBouton variante="primaire" touche="⌘S">
              Enregistrer
            </FauxBouton>
          </span>
        }
      />
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden border-t border-bd md:flex-row">
        <Colonne className="hidden w-[196px] flex-none lg:flex">
          <span className="caps">Offre visée</span>
          <FauxSegment options={["Une candidature", "Texte collé"]} choisi="Une candidature" />
          <span className="flex items-center gap-[7px] rounded-r7 bg-panel px-[7px] py-[6px] shadow-[inset_2px_0_0_var(--ac)]">
            <Avatar initiales="AN" teinte="av1" />
            <span className="flex min-w-0 flex-col">
              <span className="truncate font-medium">Designer produit senior</span>
              <span className="truncate text-[10px] text-tx-5">Atelier Nord · CAN-214</span>
            </span>
          </span>
          <span className="flex items-center gap-[7px] px-[7px] py-[6px]">
            <Avatar initiales="SH" teinte="av3" />
            <span className="flex min-w-0 flex-col">
              <span className="truncate">Product designer</span>
              <span className="truncate text-[10px] text-tx-5">Studio Halage · CAN-207</span>
            </span>
          </span>
          <span className="mt-2 flex justify-between">
            <span className="caps">Ce que l&apos;IA peut utiliser</span>
            <span className="font-mono text-[10px] text-tx-5">5 / 6</span>
          </span>
          {SECTIONS_AUTORISEES.map((s) => (
            <span key={s.libelle} className="flex h-5 items-center gap-2">
              <FauxSwitch actif={s.actif} />
              <span className={cn("flex-1", s.actif ? "text-tx-2" : "text-tx-5")}>{s.libelle}</span>
              <span className="font-mono text-[10px] text-tx-5">{s.nombre}</span>
            </span>
          ))}
          <span className="caps mt-2">Ton</span>
          <FauxSegment options={["Sobre", "Professionnel", "Direct"]} choisi="Professionnel" />
        </Colonne>

        <Bureau className="h-[400px] md:h-auto">
          <div className="flex h-[467px] w-[330px] flex-none flex-col gap-[9px] rounded-r2 bg-paper px-[22px] py-[22px] font-plex text-[7.4px] leading-[1.45] text-paper-ink shadow-sheet">
            <span className="flex items-end justify-between">
              <span className="flex flex-col gap-[2px]">
                <span className="text-[15px] font-semibold tracking-[-0.01em]">{PERSONA.nom}</span>
                <span className="text-[8.5px] text-paper-ink-2">{PERSONA.titre}</span>
              </span>
              <span className="text-right font-mono text-[6.2px] leading-[1.6] text-paper-ink-3">
                {PERSONA.ville}
                <br />
                {PERSONA.courriel}
                <br />
                {PERSONA.telephone}
              </span>
            </span>
            <span className="h-px bg-paper-rule" />
            <LigneFeuille titre="Profil">
              <span className="text-paper-ink-2">
                Designer produit depuis six ans, spécialisée dans les outils métier. Conçoit des
                parcours complexes, anime la recherche utilisateur et fait vivre un design system
                partagé par plusieurs équipes.
              </span>
            </LigneFeuille>
            <LigneFeuille titre="Expériences">
              <span className="flex flex-col gap-[7px]">
                <Poste
                  intitule="Designer produit — Parcelle"
                  periode="2022 → aujourd'hui"
                  puces={[
                    "Refonte du design system : 140 composants documentés, adoptés par 4 équipes.",
                    "Entretiens mensuels avec les exploitants pour arbitrer la feuille de route.",
                    "Planification des tournées : temps de saisie réduit de 35 %.",
                  ]}
                />
                <Poste
                  intitule="Designer UX — Agence Méridienne"
                  periode="2019 → 2022"
                  puces={["Parcours de souscription pour trois clients de l'assurance.", "Tests utilisateurs et prototypes haute fidélité."]}
                />
                <Poste
                  intitule="Designer d'interface — Studio Oblique"
                  periode="2018 → 2019"
                  puces={["Applications mobiles culturelles, de la maquette à la recette."]}
                />
              </span>
            </LigneFeuille>
            <LigneFeuille titre="Projets">
              <span className="text-paper-ink-2">
                <span className="bg-paper-mark font-semibold text-paper-ink shadow-[0_0_0_1.5px_var(--paper-mark)]">
                  Refonte du design system Parcelle
                </span>{" "}
                — jetons, documentation, gouvernance.
              </span>
            </LigneFeuille>
            <LigneFeuille titre="Compétences">
              <span className="text-paper-ink-2">
                Design system · Recherche utilisateur · Prototypage · Figma · Ateliers de cadrage ·
                Rédaction UX · Mesure d&apos;usage
              </span>
            </LigneFeuille>
            <LigneFeuille titre="Formation">
              <span className="flex justify-between">
                <span>
                  <span className="font-semibold">Master Design d&apos;interaction</span> — Lyon
                </span>
                <span className="font-mono text-[6px] text-paper-ink-3">2018</span>
              </span>
            </LigneFeuille>
            <LigneFeuille titre="Langues">
              <span className="text-paper-ink-2">Français, langue maternelle · Anglais, courant (C1)</span>
            </LigneFeuille>
          </div>
        </Bureau>

        <Colonne className="border-t border-bd md:w-[232px] md:flex-none md:border-l md:border-t-0">
          <span className="hidden flex-col gap-[10px] md:flex">
            <Etapes etapes={ETAPES_CV} />
            <span className="h-px bg-bd" />
          </span>
          <span className="flex items-baseline gap-[6px]">
            <span className="serif-title text-[24px] text-st-g">84</span>
            <span className="text-tx-4">/ 100 · score ATS</span>
          </span>
          <span className="flex flex-col gap-[5px]">
            <span className="flex justify-between">
              <span className="text-tx-4">Place sur la page</span>
              <span className="font-medium">Espace disponible</span>
            </span>
            <span className="h-1 rounded-r2 bg-chip">
              <span className="block h-1 w-[78%] rounded-r2 bg-st-g" />
            </span>
          </span>
          <span className="caps mt-1">Recommandé pour cette offre</span>
          <Proposition
            titre="Ajouter le projet « Refonte du design system »"
            gain="+4"
            detail="Très pertinent · l'offre cite le design system trois fois."
            actions={<FauxBouton variante="primaire" taille="petit">Ajouté ✓</FauxBouton>}
          />
          <span className="hidden md:contents">
            <Proposition
              titre="Mettre en avant « Tests utilisateurs »"
              gain="+2"
              detail="Pertinent · demandé dans les missions."
              actions={
                <>
                  <FauxBouton taille="petit">Ajouter</FauxBouton>
                  <FauxBouton variante="discret" taille="petit">Ignorer</FauxBouton>
                </>
              }
            />
            <span className="caps mt-1">Compétences manquantes à vérifier</span>
            <span className="flex items-center gap-[7px]">
              <StatusGlyph ton="c" petit />
              Accessibilité (RGAA)
            </span>
          </span>
        </Colonne>
      </div>
      <BarreEtat
        gauche="généré en 16,2 s · 1 180 tokens · ministral-3:3b · sur votre ordinateur"
        touches={[
          { libelle: "Enregistrer", touche: "⌘S", accent: true },
          { libelle: "Fermer", touche: "Échap" },
        ]}
      />
    </Fenetre>
  );
}

/** Analyse face à l'offre : ce qui est comparé, score calculé et son détail, puis
 *  chaque exigence avec sa preuve citée. */
export function AnalyseOffre() {
  return (
    <Fenetre hauteur="md:h-[600px]">
      <BarreTitre
        surcouche
        fil={["Documents", "Analyse face à l'offre"]}
        droite={
          <span className="flex gap-[6px]">
            <FauxBouton visible="sm">Exporter l&apos;analyse</FauxBouton>
            <FauxBouton variante="primaire">Corriger le CV</FauxBouton>
          </span>
        }
      />
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden border-t border-bd md:flex-row">
        <Colonne className="md:w-[230px] md:flex-none">
          <span className="caps hidden md:block">Ce qui est comparé</span>
          <span className="hidden items-center gap-2 rounded-r7 border border-bd bg-panel px-2 py-[7px] md:flex">
            <span className="h-[15px] w-3 flex-none rounded-r2 border border-tx-6" />
            <span className="flex flex-col">
              <span className="font-medium">CV — Designer produit</span>
              <span className="text-[10px] text-tx-5">votre CV · v2</span>
            </span>
          </span>
          <span className="hidden pl-[10px] font-mono text-[10px] text-tx-5 md:block">face à</span>
          <span className="flex items-center gap-2 rounded-r7 border border-bd bg-panel px-2 py-[7px]">
            <Avatar initiales="AN" teinte="av1" />
            <span className="flex flex-col">
              <span className="font-medium">Designer produit senior</span>
              <span className="text-[10px] text-tx-5">Atelier Nord · CAN-214</span>
            </span>
          </span>
          <span className="flex items-baseline gap-[6px] md:mt-2">
            <span className="serif-title text-[32px] leading-none text-ac-tx">78</span>
            <span className="text-tx-4">/ 100</span>
          </span>
          <span className="grid grid-cols-[repeat(20,minmax(0,1fr))] gap-[2px]">
            {Array.from({ length: 20 }, (_, i) => (
              <span key={i} className={cn("h-[3px]", i < 16 ? "bg-ac" : i < 18 ? "bg-tint-ac-bg" : "bg-chip")} />
            ))}
          </span>
          <span className="text-[10.5px] leading-[1.4] text-tx-4">
            Jusqu&apos;à 91 si vous appliquez les trois corrections listées.
          </span>
          <span className="hidden flex-col gap-[10px] md:flex">
            <span className="caps mt-[6px]">Détail du score</span>
            {DETAIL_SCORE.map((d) => (
              <span key={d.libelle} className="flex flex-col gap-1">
                <span className="flex justify-between">
                  <span>{d.libelle}</span>
                  <span className="font-mono text-[10px] text-tx-4">{d.valeur}</span>
                </span>
                <span className="h-[3px] rounded-r2 bg-chip">
                  <span className={cn("block h-[3px] rounded-r2", BARRE[d.ton])} style={{ width: `${d.part}%` }} />
                </span>
              </span>
            ))}
          </span>
        </Colonne>
        <div className="flex min-w-0 flex-1 flex-col gap-1 bg-panel px-4 py-4 md:px-6 md:py-5">
          <span className="serif-title text-[18px] md:text-[20px]">4 exigences couvertes sur 7</span>
          <span className="mb-[10px] text-[12px] leading-[1.5] text-tx-4">
            Candilog a lu l&apos;annonce ligne par ligne et cherché dans votre CV ce qui y répond.
            Chaque exigence est citée avec sa preuve — ou l&apos;absence de preuve.
          </span>
          <span className="flex justify-between pb-[6px]">
            <span className="caps">Exigences de l&apos;offre</span>
            <span className="hidden gap-[10px] text-[10.5px] text-tx-4 sm:flex">
              <span className="flex items-center gap-1"><StatusGlyph ton="g" petit />couverte</span>
              <span className="flex items-center gap-1"><StatusGlyph ton="a" petit />partielle</span>
              <span className="flex items-center gap-1"><StatusGlyph ton="c" petit />absente</span>
            </span>
          </span>
          {EXIGENCES.map((e, i) => (
            <span
              key={e.exigence}
              className={cn("flex flex-col gap-[3px] border-t border-bd-soft py-[7px]", i > 3 && "hidden md:flex")}
            >
              <span className="flex items-center gap-[9px]">
                <StatusGlyph ton={e.ton} />
                <span className="flex-1 text-[12.5px] font-medium">{e.exigence}</span>
                <Pastille teinte={TEINTE_ETAT[e.etat]}>{e.etat}</Pastille>
              </span>
              <span className="flex gap-[9px] pl-5 text-[11.5px] leading-[1.45]">
                <span className="hidden w-[92px] flex-none pt-px font-mono text-[9.5px] text-tx-5 sm:inline">{e.source}</span>
                <span className="italic text-tx-3">{e.preuve}</span>
              </span>
            </span>
          ))}
        </div>
      </div>
      <BarreEtat
        gauche="claude-sonnet · analyse distante · 7 s · CV v2 face à CAN-214"
        touches={[{ libelle: "Fermer", touche: "Échap" }]}
      />
    </Fenetre>
  );
}

/** Générateur de lettre : feuille A4 à colonne d'identité, corrections cumulées,
 *  étapes, adéquation et recommandations appuyées sur un fait du profil. */
export function GenerateurLettre() {
  return (
    <Fenetre hauteur="md:h-[580px]">
      <BarreTitre
        surcouche
        fil={["Documents", "Générer une lettre"]}
        droite={
          <span className="flex gap-[6px]">
            <FauxBouton visible="sm">Exporter en PDF</FauxBouton>
            <FauxBouton variante="primaire" touche="⌘S">
              Enregistrer
            </FauxBouton>
          </span>
        }
      />
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden border-t border-bd md:flex-row">
        <Bureau className="h-[380px] flex-col items-center md:h-auto">
          <div className="grid h-[400px] w-[min(420px,calc(100%-24px))] flex-none grid-cols-[96px_1fr] overflow-hidden rounded-r2 bg-paper font-plex text-[7.6px] leading-[1.55] text-paper-ink shadow-sheet sm:grid-cols-[108px_1fr]">
            <span className="flex flex-col gap-[10px] bg-paper-side px-3 py-[22px]">
              <span className="text-[11px] font-semibold leading-[1.2]">{PERSONA.nom}</span>
              <span className="text-paper-ink-2">{PERSONA.titre}</span>
              <span className="break-all font-mono text-[6.2px] leading-[1.7] text-paper-ink-3">
                {PERSONA.ville}
                <br />
                {PERSONA.telephone}
                <br />
                {PERSONA.courriel}
              </span>
              <span className="flex-1" />
              <span className="font-mono text-[6px] text-paper-ink-3">Pièce jointe : curriculum vitæ</span>
            </span>
            <span className="flex flex-col gap-[7px] px-5 py-[22px] text-paper-ink">
              <span className="flex flex-col items-end gap-px text-paper-ink-2">
                <span className="font-semibold text-paper-ink">Atelier Nord</span>
                <span>À l&apos;attention de l&apos;équipe recrutement</span>
                <span>Lyon, le 23 septembre 2026</span>
              </span>
              <span className="font-semibold">Objet : candidature au poste de designer produit senior</span>
              <span>Madame, Monsieur,</span>
              <span>
                Depuis quatre ans, je conçois chez Parcelle les outils qu&apos;utilisent chaque jour des
                équipes logistiques : planification, suivi des tournées, tableaux de bord. Votre
                annonce décrit exactement ce passage à l&apos;échelle : un produit qui grandit sans
                perdre sa cohérence.
              </span>
              <span className="bg-paper-mark shadow-[0_0_0_2px_var(--paper-mark)]">
                J&apos;ai piloté la refonte de notre design system, aujourd&apos;hui adopté par quatre
                équipes, et j&apos;organise chaque mois des entretiens avec les exploitants pour
                arbitrer les priorités avec le produit.
              </span>
              <span>
                Rejoindre Atelier Nord, c&apos;est pour moi poursuivre ce travail sur des outils où la
                clarté fait gagner du temps à ceux qui les utilisent.
              </span>
            </span>
          </div>
          <span className="mt-3 hidden w-[420px] flex-col gap-[7px] rounded-r9 border border-bd bg-panel px-[10px] py-[9px] md:flex">
            <span className="flex justify-between text-[10.5px]">
              <span className="caps">Corrections</span>
              <span className="text-tx-5">1 consigne appliquée</span>
            </span>
            <span className="flex flex-wrap gap-[5px]">
              <Pastille teinte="ac">Plus court ✓</Pastille>
              <Pastille>Moins formel</Pastille>
              <Pastille>Ajoute un chiffre</Pastille>
              <Pastille>Cite l&apos;entreprise</Pastille>
            </span>
          </span>
        </Bureau>
        <Colonne className="border-t border-bd md:w-[240px] md:flex-none md:border-l md:border-t-0">
          <span className="hidden flex-col gap-[10px] md:flex">
            <Etapes etapes={ETAPES_LETTRE} />
            <span className="h-px bg-bd" />
          </span>
          <span className="flex items-baseline gap-[6px]">
            <span className="serif-title text-[24px] text-st-a">68</span>
            <span className="flex-1 text-tx-4">/ 100 · adéquation</span>
            <span className="font-mono text-[10px] text-tx-5">jusqu&apos;à 84</span>
          </span>
          <span className="flex h-1 rounded-r2 bg-chip">
            <span className="w-[68%] rounded-l-r2 bg-st-a" />
            <span className="w-[16%] bg-tint-ac-bg" />
          </span>
          <span className="text-[10.5px] leading-[1.4] text-tx-4">
            Mesure la part de la lettre qui parle vraiment de cette offre.
          </span>
          <span className="mt-1 flex justify-between">
            <span className="caps">Recommandations</span>
            <span className="font-mono text-[10px] text-st-a">2 à traiter</span>
          </span>
          <Proposition
            glyphe
            titre="Aborder « mesure d'impact »"
            gain="+9"
            detail="Votre profil : « Planification des tournées — temps de saisie réduit de 35 % »."
            actions={
              <>
                <FauxBouton variante="primaire" taille="petit">Appliquer</FauxBouton>
                <FauxBouton taille="petit">Ignorer</FauxBouton>
              </>
            }
          />
          <span className="hidden md:contents">
            <Proposition
              glyphe
              titre="Aborder « collaboration avec les développeurs »"
              gain="+7"
              detail="Votre profil : « Jetons, documentation, gouvernance »."
            />
            <span className="caps mt-1">Absent de votre profil</span>
            <span className="flex items-center gap-[7px] text-tx-3">
              <StatusGlyph ton="n" petit />
              Accessibilité (RGAA)
            </span>
          </span>
        </Colonne>
      </div>
      <BarreEtat
        gauche="rédigée en 13,3 s · score 68/100 · 2 recommandations à traiter"
        touches={[
          { libelle: "Enregistrer", touche: "⌘S", accent: true },
          { libelle: "Fermer", touche: "Échap" },
        ]}
      />
    </Fenetre>
  );
}
