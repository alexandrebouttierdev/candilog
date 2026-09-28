import { LineIcon } from "@/components/ui/LineIcon";
import { Avatar, FauxBouton, Filet, Kbd, Pastille, StatusGlyph } from "@/components/landing/app/primitives";
import { cn } from "@/lib/cn";
import { CANDIDATURES, LIBELLE_STATUT, candidaturesParStatut, type Candidature } from "@/lib/data/demo";

/* Liste groupée par statut (`ApplicationsPage`), filtrée sur « Statut n'est pas Refusée ». */
const GROUPES = (["n", "a", "g"] as const).map((ton) => ({ ton, lignes: candidaturesParStatut(ton) }));
const CHOISIE = "CAN-207";

function Ligne({ c }: { c: Candidature }) {
  const choisie = c.ref === CHOISIE;
  return (
    <div
      className={cn(
        "grid h-[34px] items-center gap-3 border-b border-bd-soft px-3 text-[12.5px] md:h-[32px] md:px-[14px]",
        "grid-cols-[18px_minmax(0,1fr)_40px] sm:grid-cols-[58px_minmax(0,2.2fr)_minmax(0,1.5fr)_52px_44px] xl:grid-cols-[14px_58px_minmax(0,2.2fr)_minmax(0,1.5fr)_52px_78px_44px]",
        choisie && "bg-sel",
      )}
    >
      <span className="hidden size-[11px] rounded-r3 border border-bd-menu bg-panel xl:block" />
      <span className="hidden font-mono text-[10.5px] text-tx-5 sm:inline">{c.ref}</span>
      <Avatar initiales={c.initiales} teinte={c.avatar} affichage="sous-sm" />
      <span className={cn("truncate text-tx", choisie && "font-medium")}>
        {c.poste}
        <span className="text-tx-5 sm:hidden"> · {c.entreprise}</span>
      </span>
      <span className="hidden min-w-0 items-center gap-[7px] text-tx-3 sm:flex">
        <Avatar initiales={c.initiales} teinte={c.avatar} />
        <span className="truncate">{c.entreprise}</span>
      </span>
      <span className="hidden sm:block">
        <Pastille>{c.contrat}</Pastille>
      </span>
      <span className="hidden truncate text-tx-4 xl:inline">{c.ville}</span>
      <span className="text-right font-mono text-[10.5px] text-tx-5">{c.envoyee}</span>
    </div>
  );
}

function Champ({ libelle, children }: { libelle: string; children: React.ReactNode }) {
  return (
    <span className="grid h-[26px] grid-cols-[78px_1fr] items-center">
      <span className="text-tx-4">{libelle}</span>
      {children}
    </span>
  );
}

/** Fiche latérale d'une candidature (inspecteur 300 px, dès 1280 px). */
export function FicheCandidature({ c }: { c: Candidature }) {
  return (
    <div className="flex flex-col gap-[14px]">
      <span className="flex flex-col gap-1">
        <span className="serif-title text-[19px]">{c.poste}</span>
        <span className="text-[12px] text-tx-4">
          {c.entreprise} · {c.ville}
        </span>
      </span>
      <span className="flex flex-col gap-px text-[12px]">
        <span className="grid h-[28px] grid-cols-[78px_1fr] items-center">
          <span className="text-tx-4">Statut</span>
          <span className="flex items-center gap-[7px] rounded-r6 bg-elev px-2 py-1">
            <StatusGlyph ton={c.statut} />
            {LIBELLE_STATUT[c.statut]}
            <span className="flex-1" />
            <Kbd>S</Kbd>
          </span>
        </span>
        <Champ libelle="Contrat">{c.contrat} · 35 h</Champ>
        <Champ libelle="Type">{c.type}</Champ>
        <Champ libelle="Envoyée">
          <span className="font-mono text-[11px]">{c.anciennete}</span>
        </Champ>
        <Champ libelle={c.prochain.libelle}>
          <span className="font-mono text-[11px] text-ac-tx">{c.prochain.valeur}</span>
        </Champ>
      </span>
      <Filet />
      <span className="flex flex-col gap-2 text-[12px]">
        <span className="caps">Documents</span>
        {(c.documents.length > 0 ? c.documents : ["Aucun document rattaché"]).map((d) => (
          <span key={d} className="flex items-center gap-2">
            <span className="h-[11px] w-[9px] rounded-r2 border border-tx-6" />
            {d}
          </span>
        ))}
      </span>
      <Filet />
      <span className="flex flex-col gap-2 text-[12px]">
        <span className="caps">Activité</span>
        {c.activite.map(({ date, fait }) => (
          <span key={date + fait} className="flex gap-[10px]">
            <span className="font-mono text-[10.5px] text-tx-5">{date}</span>
            {fait}
          </span>
        ))}
      </span>
    </div>
  );
}

/** Écran Candidatures, vue Liste (`screens/02-applications-list.png`). */
export function EcranListe() {
  const choisie = CANDIDATURES.find((c) => c.ref === CHOISIE);
  return (
    <>
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex h-[38px] flex-none items-center gap-2 border-b border-bd-soft px-3 text-[12px]">
          <span className="flex items-center gap-[6px] whitespace-nowrap rounded-r6 bg-tint-ac-bg px-2 py-[3px] text-tint-ac-tx">
            Statut <b className="font-medium">n&apos;est pas</b> Refusée <span className="opacity-60">✕</span>
          </span>
          <span className="hidden whitespace-nowrap text-tx-4 sm:inline">
            + Filtre <Kbd>F</Kbd>
          </span>
          <span className="flex-1" />
          <span className="font-mono text-[10.5px] text-tx-5">11 / 14</span>
          <span className="hidden items-center gap-[6px] px-[6px] text-tx-4 lg:flex">
            <LineIcon name="search" size={13} />
            Rechercher <Kbd>/</Kbd>
          </span>
          <FauxBouton visible="lg">CSV</FauxBouton>
          <FauxBouton variante="primaire" touche="N" visible="md">
            Ajouter une candidature
          </FauxBouton>
        </div>
        {GROUPES.map(({ ton, lignes }) => (
          <div key={ton} className="flex flex-col">
            <span className="flex h-[28px] items-center gap-[9px] bg-group px-3 text-[12px] font-medium md:px-[14px]">
              <StatusGlyph ton={ton} />
              {LIBELLE_STATUT[ton]}
              <span className="font-mono text-[10.5px] font-normal text-tx-5">{lignes.length}</span>
            </span>
            {lignes.map((c) => (
              <Ligne key={c.ref} c={c} />
            ))}
          </div>
        ))}
      </div>
      {choisie ? (
        <div className="hidden w-[300px] flex-none flex-col gap-[14px] border-l border-bd px-[18px] py-4 xl:flex">
          <span className="flex justify-between">
            <span className="font-mono text-[10.5px] text-tx-5">{choisie.ref}</span>
            <span className="flex gap-[6px]">
              <Pastille>↗</Pastille>
              <Pastille>⋯</Pastille>
              <Pastille>✕</Pastille>
            </span>
          </span>
          <FicheCandidature c={choisie} />
        </div>
      ) : null}
    </>
  );
}
