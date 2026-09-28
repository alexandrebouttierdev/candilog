import { BarreEtat, BarreTitre, Accessoire, Fenetre, Navigation, Panneau } from "@/components/landing/app/Fenetre";
import { FauxBouton, Filet, Pastille, StatusGlyph } from "@/components/landing/app/primitives";
import { cn } from "@/lib/cn";
import { AUJOURDHUI, SANS_REPONSE, STATUTS, candidaturesParStatut, type LigneJour } from "@/lib/data/demo";

/* Largeurs des quatre segments de la barre « Situation », proportionnelles aux
   décomptes 5 / 3 / 3 / 3 : même calcul que l'application, sur les mêmes données. */
const SEGMENT = { n: "bg-st-n", a: "bg-st-a", g: "bg-st-g", c: "bg-st-c" } as const;

function Ligne({ ligne, serree }: { ligne: LigneJour; serree: boolean }) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 border-b border-bd-soft px-3 md:px-[14px]",
        serree ? "min-h-[40px] py-2" : "min-h-[50px] py-2",
      )}
    >
      <StatusGlyph ton={ligne.ton} />
      <span className="hidden w-[56px] flex-none font-mono text-[10.5px] text-tx-5 sm:inline">{ligne.ref}</span>
      <span className="flex min-w-0 flex-1 flex-col gap-[2px]">
        <span className="truncate text-[13px] text-tx">{ligne.titre}</span>
        <span className={cn("truncate text-[11.5px] text-tx-5", ligne.jour && "md:hidden")}>{ligne.detail}</span>
      </span>
      {ligne.jour ? (
        <>
          <span className="hidden whitespace-nowrap text-[12px] text-tx-4 md:inline">{ligne.detail}</span>
          <span className="w-[52px] flex-none whitespace-nowrap text-right font-mono text-[10.5px] text-tx-5">{ligne.jour}</span>
        </>
      ) : null}
      {ligne.pastille ? (
        <Pastille teinte={ligne.pastille.teinte} mono>
          {ligne.pastille.texte}
        </Pastille>
      ) : null}
      {ligne.actionSecondaire ? (
        <FauxBouton visible="md">{ligne.actionSecondaire}</FauxBouton>
      ) : null}
      {ligne.action ? (
        <FauxBouton
          variante={ligne.action.primaire ? "primaire" : "secondaire"}
          touche={ligne.action.touche}
          visible="sm"
        >
          {ligne.action.libelle}
        </FauxBouton>
      ) : null}
    </div>
  );
}

function Situation() {
  return (
    <div className="hidden w-[290px] flex-none flex-col gap-[22px] border-l border-bd-soft px-[22px] py-[26px] xl:flex">
      <div className="flex flex-col gap-3">
        <span className="caps">Situation</span>
        <div className="flex h-1 gap-[3px]">
          {STATUTS.map(({ ton }) => (
            <span
              key={ton}
              className={cn("rounded-r2", SEGMENT[ton])}
              style={{ flexGrow: candidaturesParStatut(ton).length }}
            />
          ))}
        </div>
        {STATUTS.map(({ ton, libelle }) => (
          <span key={ton} className="flex items-center gap-[9px] text-[12.5px] text-tx-2">
            <StatusGlyph ton={ton} petit />
            <span className="flex-1">{libelle}</span>
            <span className="font-mono text-[10.5px] text-tx-5">{candidaturesParStatut(ton).length}</span>
          </span>
        ))}
      </div>
      <Filet />
      <div className="flex flex-col gap-3">
        <span className="caps">30 derniers jours</span>
        <div className="flex gap-[26px]">
          {[
            ["9", "envoyées"],
            ["4", "réponses"],
            ["2", "entretiens"],
          ].map(([valeur, libelle]) => (
            <span key={libelle} className="flex flex-col gap-1">
              <span className="serif-title text-[25px] leading-none">{valeur}</span>
              <span className="text-[11.5px] text-tx-4">{libelle}</span>
            </span>
          ))}
        </div>
      </div>
      <Filet />
      <div className="flex flex-col gap-[11px]">
        <span className="flex justify-between">
          <span className="caps">Sans réponse</span>
          <span className="font-mono text-[10.5px] text-st-c">{SANS_REPONSE.length}</span>
        </span>
        {SANS_REPONSE.map(({ ref, entreprise, jours }) => (
          <span key={ref} className="flex items-center gap-[10px] text-[12.5px]">
            <span className="font-mono text-[10px] text-tx-6">{ref}</span>
            <span className="flex-1 text-tx-2">{entreprise}</span>
            <span className="font-mono text-[10.5px] text-st-c">{jours}</span>
          </span>
        ))}
      </div>
    </div>
  );
}

/** Écran « Aujourd'hui » (`reference_design/screens/01-today-light.png`). */
export function EcranAujourdhui() {
  return (
    <Fenetre hauteur="md:h-[640px] lg:h-[660px]">
      <BarreTitre fil={["Aujourd'hui", "Lundi 28 septembre"]} droite={<Accessoire>28 septembre 2026</Accessoire>} />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <Navigation active="today" />
        <Panneau>
          <div className="flex min-w-0 flex-1 flex-col pb-2 pt-5 md:px-[22px] md:pt-[26px]">
            <div className="flex flex-col gap-1 px-3 pb-4 sm:flex-row sm:items-baseline sm:gap-[14px] md:px-1 md:pb-[18px] [&>span]:whitespace-nowrap">
              <span className="serif-title text-[21px] md:text-[23px]">Lundi 28 septembre</span>
              <span className="font-mono text-[10.5px] text-tx-5 md:text-[11px]">1 en retard · 2 aujourd&apos;hui · 3 cette semaine</span>
            </div>
            {AUJOURDHUI.map((groupe) => (
              <div key={groupe.libelle} className="flex flex-col">
                <span className="flex h-[30px] items-center gap-[9px] bg-group px-3 text-[12.5px] font-medium md:px-[14px]">
                  <StatusGlyph ton={groupe.ton} />
                  {groupe.libelle}
                  <span className="font-mono text-[10.5px] font-normal text-tx-5">{groupe.lignes.length}</span>
                </span>
                {groupe.lignes.map((ligne) => (
                  <Ligne key={ligne.ref + ligne.titre} ligne={ligne} serree={groupe.libelle === "Cette semaine"} />
                ))}
              </div>
            ))}
          </div>
          <Situation />
        </Panneau>
      </div>
      <BarreEtat
        gauche="6 échéances · prochaine aujourd'hui à 14:30"
        touches={[
          { libelle: "Faire", touche: "⏎" },
          { libelle: "Reporter", touche: "R" },
          { libelle: "Actions", touche: "⌘K", accent: true },
        ]}
      />
    </Fenetre>
  );
}
