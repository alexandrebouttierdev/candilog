import { BarreEtat, BarreTitre, Accessoire, Fenetre, Navigation, Panneau } from "@/components/landing/app/Fenetre";
import { FauxBouton, Pastille, Point, StatusGlyph } from "@/components/landing/app/primitives";
import { EnTeteSection } from "@/components/landing/EnTeteSection";
import { BrandIcon } from "@/components/ui/BrandIcon";
import { BrandMark } from "@/components/ui/BrandMark";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { FOURNISSEURS_APERCU, ROUTAGE } from "@/lib/data/demo";

const JAUGES = [
  ["Confidentialité", "Rien ne sort"],
  ["Coût", "Gratuit"],
  ["Hors connexion", "Oui"],
] as const;

const GARANTIES = [
  {
    repere: "local · distant",
    titre: "Vous voyez ce qui sort",
    texte:
      "Chaque tâche indique si elle reste sur la machine ou part chez un fournisseur. Le premier envoi à un service distant demande votre accord.",
  },
  {
    repere: "étape 2 / 4 · 00:08",
    titre: "Une progression honnête",
    texte:
      "L'étape en cours et le temps écoulé, jamais un pourcentage inventé. Vous arrêtez un traitement quand vous le décidez, et la durée réelle s'affiche à la fin.",
  },
  {
    repere: "profil → document",
    titre: "Rien d'inventé",
    texte:
      "Les documents générés sont recadrés sur les faits réels de votre profil. Les textes libres du modèle sont signalés comme tels.",
  },
] as const;

/** Écran IA (`screens/13-ai-who-does-what.png`) : fournisseurs, IA locale, « Qui fait quoi ». */
function EcranIa() {
  return (
    <Fenetre hauteur="md:h-[620px]">
      <BarreTitre fil={["Intelligence artificielle", "IA locale"]} droite={<Accessoire>IA locale · 2 modèles installés</Accessoire>} />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <Navigation
          active="ai"
          pied={
            <span className="hidden h-[26px] items-center gap-2 px-3 text-[11.5px] text-tx-3 lg:flex">
              <span className="size-[6px] rounded-full bg-st-g" />
              <span className="flex-1 whitespace-nowrap">Routage IA</span>
              <span className="font-mono text-[10px] text-tx-6">3 loc · 1 dist</span>
            </span>
          }
        />
        <Panneau>
          <div className="hidden w-[200px] flex-none flex-col gap-[3px] border-r border-bd-soft px-[10px] py-4 text-[12px] xl:flex">
            <span className="caps px-[6px] pb-2">Fournisseurs</span>
            {FOURNISSEURS_APERCU.map((f) => (
              <span key={f.nom} className={cn("flex items-center gap-[9px] rounded-r7 px-2 py-[7px]", f.choisi && "bg-elev")}>
                {f.logo ? (
                  <span className="grid size-[22px] flex-none place-items-center rounded-r6 bg-chip text-tx-2">
                    <BrandIcon name={f.logo} set="providers" size={13} />
                  </span>
                ) : (
                  <BrandMark size={22} />
                )}
                <span className="flex min-w-0 flex-1 flex-col">
                  <span className="font-medium">{f.nom}</span>
                  <span className="text-[10.5px] text-tx-5">{f.detail}</span>
                </span>
                <Point ou={f.pret ? "local" : "aucun"} />
              </span>
            ))}
            <span className="p-2 text-tx-4">+ Ajouter un fournisseur</span>
          </div>
          <div className="flex min-w-0 flex-1 flex-col gap-[14px] px-4 py-5 md:px-[26px]">
            <span className="flex items-center justify-between">
              <span className="serif-title text-[21px]">IA locale</span>
              <FauxBouton touche="T" visible="sm">Tester la connexion</FauxBouton>
            </span>
            <span className="max-w-[560px] text-[12.5px] leading-[1.55] text-tx-3">
              Les modèles tournent sur votre ordinateur : vos CV, lettres et candidatures n&apos;en
              sortent pas. Gratuit et utilisable hors connexion ; en contrepartie, c&apos;est plus
              lent, et la qualité dépend du modèle installé.
            </span>
            <span className="hidden gap-10 text-[11.5px] sm:flex">
              {JAUGES.map(([libelle, valeur]) => (
                <span key={libelle} className="flex flex-col gap-[5px]">
                  <span className="flex gap-[3px]">
                    <span className="h-[3px] w-4 bg-st-g" />
                    <span className="h-[3px] w-4 bg-st-g" />
                    <span className="h-[3px] w-4 bg-st-g" />
                  </span>
                  <span className="font-medium">{libelle}</span>
                  <span className="text-tx-5">{valeur}</span>
                </span>
              ))}
            </span>
            <span className="mt-1 hidden justify-between md:flex">
              <span className="caps">Modèles installés</span>
              <span className="text-[11px] text-ac-tx">Installer un modèle</span>
            </span>
            <span className="hidden flex-col text-[12.5px] md:flex">
              <span className="flex h-[40px] items-center gap-[10px] rounded-r7 bg-sel px-[10px] shadow-[inset_0_0_0_1px_var(--tint-ac-bg)]">
                <StatusGlyph ton="g" />
                <span className="flex flex-1 flex-col">
                  <span className="font-medium">Ministral 3 · 3B</span>
                  <span className="text-[10.5px] text-tx-5">Rapide, léger</span>
                </span>
                <Pastille mono>3 B</Pastille>
                <span className="text-[11.5px] text-ac-tx">3 tâches</span>
              </span>
              <span className="flex h-[40px] items-center gap-[10px] px-[10px]">
                <StatusGlyph ton="n" />
                <span className="flex flex-1 flex-col">
                  <span className="font-medium">Ministral 3 · 8B</span>
                  <span className="text-[10.5px] text-tx-5">Plus fin sur les CV longs · peut être lent</span>
                </span>
                <Pastille mono>8 B</Pastille>
                <FauxBouton>Assigner…</FauxBouton>
              </span>
            </span>
            <span className="mt-1 flex justify-between gap-3">
              <span className="caps">Qui fait quoi</span>
              <span className="hidden text-[11px] text-tx-6 sm:inline">Chaque tâche peut utiliser un modèle différent</span>
            </span>
            <span className="flex flex-col">
              {ROUTAGE.map((r) => (
                <span key={r.tache} className="flex min-h-[33px] items-center gap-[10px] border-b border-bd-soft py-1 text-[12.5px]">
                  <span className="flex-1 text-tx-2">{r.tache}</span>
                  <span
                    className={cn(
                      "flex items-center gap-[7px] whitespace-nowrap rounded-r6 bg-group px-[9px] py-1 text-[12px]",
                      r.ou === "aucun" ? "text-tx-5" : "text-tx-2",
                    )}
                  >
                    <Point ou={r.ou} />
                    {r.modele}
                    <span className="text-tx-6">›</span>
                  </span>
                </span>
              ))}
            </span>
          </div>
        </Panneau>
      </div>
      <BarreEtat
        gauche="5 tâches · 3 locales · 1 distante · 1 non configurée"
        touches={[
          { libelle: "Tester", touche: "T" },
          { libelle: "Actions", touche: "⌘K", accent: true },
        ]}
      />
    </Fenetre>
  );
}

/**
 * Section IA : ce que Candilog fait réellement de l'IA. Les fournisseurs cités sont
 * ceux de `src/features/settings/model/providers.ts` ; le routage est celui de
 * `AI_TASK_ROUTING.md`. Aucune promesse de qualité : la section dit où vont les
 * données et comment la progression est rapportée.
 */
export function AiSection() {
  return (
    <section id="ia" aria-labelledby="ia-titre" className="border-y border-bd bg-panel">
      <Reveal className="mx-auto flex max-w-[1200px] flex-col gap-10 px-4 py-16 md:gap-12 md:px-8 md:py-24 xl:px-0 xl:py-32">
        <EnTeteSection
          id="ia-titre"
          surTitre="Intelligence artificielle"
          titre={
            <>
              Votre modèle,
              <br className="hidden sm:block" /> tâche par tâche.
            </>
          }
        >
          Installez l&apos;IA locale depuis l&apos;application, ou connectez le fournisseur dont
          vous avez la clé : Mistral, OpenAI, Gemini, Claude, DeepSeek, ou tout service compatible
          OpenAI. Chaque tâche peut ensuite utiliser un modèle différent.
        </EnTeteSection>
        <figure className="m-0">
          <figcaption className="sr-only">
            Écran Intelligence artificielle de Candilog : l&apos;IA locale avec deux modèles
            installés, et la section « Qui fait quoi » qui attribue un modèle à chacune des cinq
            tâches — trois en local, l&apos;analyse de CV chez un fournisseur distant, une non
            configurée.
          </figcaption>
          <div aria-hidden="true" className="window-lift">
            <EcranIa />
          </div>
        </figure>
        <ul className="m-0 grid list-none gap-6 p-0 md:grid-cols-3 md:gap-10">
          {GARANTIES.map((g) => (
            <li key={g.titre} className="flex flex-col gap-2 border-t border-bd pt-5">
              <span className="font-mono text-[11px] text-tx-4">{g.repere}</span>
              <h3 className="text-[16px] font-semibold">{g.titre}</h3>
              <p className="text-pretty text-[14px] leading-[1.6] text-tx-3">{g.texte}</p>
            </li>
          ))}
        </ul>
      </Reveal>
    </section>
  );
}
