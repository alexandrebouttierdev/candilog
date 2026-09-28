import { LineIcon } from "@/components/ui/LineIcon";
import { Avatar, FauxBouton, FeuilleMiniature, Filet, Pastille, StatusGlyph } from "@/components/landing/app/primitives";
import { cn } from "@/lib/cn";
import { DOCUMENTS } from "@/lib/data/demo";

const ONGLETS = [
  ["Tous", "6"],
  ["CV", "3"],
  ["Lettres", "3"],
  ["Analyses", "2"],
] as const;

/** Écran Documents (`screens/07-resumes.png`) : liste groupée et inspecteur du CV choisi. */
export function EcranDocuments() {
  return (
    <>
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex h-[38px] flex-none items-center gap-1 border-b border-bd-soft px-3 text-[12px]">
          {ONGLETS.map(([libelle, compte], i) => (
            <span
              key={libelle}
              className={cn("whitespace-nowrap rounded-r6 px-2 py-1", i === 0 ? "bg-elev font-medium" : "text-tx-3", i > 1 && "hidden sm:inline")}
            >
              {libelle} <span className="font-mono text-[10px] text-tx-5">{compte}</span>
            </span>
          ))}
          <span className="flex-1" />
          <span className="hidden px-2 text-tx-4 lg:inline">Importer</span>
          <FauxBouton visible="md">Générer une lettre</FauxBouton>
          <FauxBouton variante="primaire" touche="N">
            Générer un CV
          </FauxBouton>
        </div>
        {DOCUMENTS.map(({ groupe, lignes }) => (
          <div key={groupe} className="flex flex-col">
            <span className="flex h-[30px] items-center gap-2 bg-group px-[14px] text-[12px] font-medium">
              <LineIcon name="documents" size={13} />
              {groupe}
              <span className="font-mono text-[10.5px] font-normal text-tx-5">{lignes.length}</span>
            </span>
            {lignes.map((d) => (
              <div
                key={d.nom}
                className={cn(
                  "flex h-[44px] items-center gap-3 border-b border-bd-soft px-[14px]",
                  "choisi" in d && d.choisi && "bg-sel",
                )}
              >
                <FeuilleMiniature className="h-[26px] w-[20px]" />
                <span className="flex min-w-0 flex-1 flex-col gap-[2px]">
                  <span className="truncate text-[13px] font-medium">{d.nom}</span>
                  <span className="truncate text-[11.5px] text-tx-5">{d.detail}</span>
                </span>
                {"pastille" in d ? (
                  <Pastille teinte={d.pastille.teinte} mono>
                    {d.pastille.texte}
                  </Pastille>
                ) : null}
                <span className="hidden w-[40px] text-right font-mono text-[10.5px] text-tx-5 sm:inline">{d.date}</span>
                <Avatar initiales={d.initiales} teinte={d.avatar} affichage="des-sm" />
              </div>
            ))}
          </div>
        ))}
      </div>
      <div className="hidden w-[300px] flex-none flex-col gap-[14px] border-l border-bd px-[18px] py-4 xl:flex">
        <span className="flex gap-3">
          <span className="flex h-[72px] w-[52px] flex-none flex-col gap-[3px] rounded-r3 border border-bd-menu bg-paper px-[6px] py-2">
            <span className="h-[3px] w-[60%] bg-tx-4" />
            <span className="h-[1.5px] bg-tx-7" />
            <span className="h-[1.5px] bg-tx-7" />
            <span className="h-[1.5px] w-[80%] bg-tx-7" />
            <span className="mt-[3px] h-[1.5px] bg-tx-7" />
            <span className="h-[1.5px] bg-tx-7" />
          </span>
          <span className="flex min-w-0 flex-col gap-1">
            <span className="text-[13.5px] font-medium">CV — Designer produit senior</span>
            <span className="text-[11.5px] text-tx-5">Atelier Nord · généré le 23-09</span>
            <span className="mt-1 flex gap-[6px]">
              <FauxBouton variante="primaire">Ouvrir</FauxBouton>
              <FauxBouton>PDF</FauxBouton>
            </span>
          </span>
        </span>
        <span className="flex flex-col gap-2">
          <span className="flex items-baseline gap-[6px]">
            <span className="serif-title text-[26px] text-st-g">84</span>
            <span className="text-[11.5px] text-tx-4">/ 100 · lisibilité par les robots de tri</span>
          </span>
          <span className="grid grid-cols-10 gap-[3px]">
            {Array.from({ length: 10 }, (_, i) => (
              <span key={i} className={cn("h-[3px]", i < 8 ? "bg-st-g" : "bg-chip")} />
            ))}
          </span>
        </span>
        <span className="flex flex-col gap-[9px] text-[12px] leading-[1.45] text-tx-2">
          <span className="flex gap-2">
            <StatusGlyph ton="g" className="mt-[3px]" />
            Les intitulés de poste reprennent ceux de l&apos;offre.
          </span>
          <span className="flex gap-2">
            <StatusGlyph ton="a" className="mt-[3px]" />
            Une expérience sur trois n&apos;a pas de résultat chiffré.
          </span>
          <span className="flex gap-2">
            <StatusGlyph ton="a" className="mt-[3px]" />
            Une exigence de l&apos;offre manque : accessibilité (RGAA).
          </span>
        </span>
        <Filet />
        <span className="flex flex-col gap-[7px] text-[12px]">
          <span className="caps">Détails</span>
          <span className="grid grid-cols-[88px_1fr]">
            <span className="text-tx-4">Candidature</span>
            <span className="text-ac-tx">Atelier Nord</span>
          </span>
          <span className="grid grid-cols-[88px_1fr]">
            <span className="text-tx-4">Modèle IA</span>
            Ministral 3 · local
          </span>
          <span className="grid grid-cols-[88px_1fr]">
            <span className="text-tx-4">Généré en</span>
            <span className="font-mono text-[11px]">16,2 s · 1 180 tokens</span>
          </span>
        </span>
        <Filet />
        <span className="flex flex-col gap-[7px] text-[12px]">
          <span className="flex justify-between">
            <span className="caps">Versions</span>
            <span className="font-mono text-[10.5px] text-tx-6">3</span>
          </span>
          <span className="flex gap-2">
            <span className="font-mono text-[10.5px] text-ac-tx">v3</span>
            <span className="flex-1">Relecture manuelle</span>
            <span className="font-mono text-[10.5px] text-tx-5">26-09</span>
          </span>
          <span className="flex gap-2 text-tx-4">
            <span className="font-mono text-[10.5px]">v2</span>
            <span className="flex-1">Projet « Refonte Parcelle » ajouté</span>
            <span className="font-mono text-[10.5px]">24-09</span>
          </span>
        </span>
      </div>
    </>
  );
}
