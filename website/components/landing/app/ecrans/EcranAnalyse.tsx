import { FauxBouton, StatusGlyph } from "@/components/landing/app/primitives";
import { cn } from "@/lib/cn";
import { CANAUX, INDICATEURS, PARCOURS_CANDIDATURES, RYTHME } from "@/lib/data/demo";

/* Graphiques dessinés avec les primitives du design, sans bibliothèque (décision D7). */
const BARRE = { n: "bg-ac", a: "bg-st-a", g: "bg-st-g", c: "bg-st-c" } as const;
const MAX_RYTHME = Math.max(...RYTHME.map((s) => s.nombre));

function Bloc({ titre, aside, children, className }: { titre: string; aside?: string; children: React.ReactNode; className?: string }) {
  return (
    <div className={cn("flex flex-col gap-3 rounded-r9 bg-group px-4 py-[14px]", className)}>
      <span className="flex justify-between gap-2">
        <span className="caps">{titre}</span>
        {aside ? <span className="text-[11px] text-tx-5">{aside}</span> : null}
      </span>
      {children}
    </div>
  );
}

/** Vue Analyse des candidatures (`screens/10-analytics.png`). Pas de flèche de tendance :
 *  il n'existe pas de période de comparaison, une variation serait inventée. */
export function EcranAnalyse() {
  return (
    <div className="flex min-w-0 flex-1 flex-col">
      <div className="flex h-[38px] flex-none items-center gap-2 border-b border-bd-soft px-3 text-[12px]">
        <span className="flex gap-[2px] rounded-r7 bg-elev p-[2px]">
          <span className="rounded-r5 px-2 py-[3px]">30 j</span>
          <span className="rounded-r5 bg-panel px-2 py-[3px] font-medium">90 j</span>
          <span className="rounded-r5 px-2 py-[3px]">Tout</span>
        </span>
        <span className="flex-1" />
        <FauxBouton>Exporter en CSV</FauxBouton>
      </div>
      <div className="flex flex-col gap-3 p-3 md:p-4">
        <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
          {INDICATEURS.map((i) => (
            <div key={i.libelle} className="flex flex-col gap-2 rounded-r9 bg-group px-4 py-[14px]">
              <span className="text-[11.5px] text-tx-4">{i.libelle}</span>
              <span className="flex items-baseline gap-[5px]">
                <span className="serif-title text-[25px] leading-none">{i.valeur}</span>
                <span className="text-[12px] text-tx-4">{i.unite}</span>
              </span>
              <span className="text-[11px] text-tx-5">{i.detail}</span>
            </div>
          ))}
        </div>
        <div className="grid gap-3 lg:grid-cols-2">
          <Bloc titre="Parcours des 14 candidatures">
            {PARCOURS_CANDIDATURES.map((p) => (
              <span key={p.libelle} className="flex flex-col gap-[6px]">
                <span className="flex items-center gap-2 text-[12.5px]">
                  <StatusGlyph ton={p.ton} />
                  <span className="flex-1">{p.libelle}</span>
                  <span className="serif-title text-[15px]">{p.nombre}</span>
                  <span className="w-9 text-right font-mono text-[10.5px] text-tx-5">{p.part}%</span>
                </span>
                <span className="h-1 rounded-r2 bg-chip">
                  <span className={cn("block h-1 rounded-r2", BARRE[p.ton])} style={{ width: `${p.part}%` }} />
                </span>
                {p.note ? <span className="text-[11px] text-tx-5">{p.note}</span> : null}
              </span>
            ))}
          </Bloc>
          <Bloc titre="Rythme d'envoi" aside="candidatures par semaine" className="hidden md:flex">
            <span className="flex h-[140px] items-end gap-[14px] border-b border-bd pt-2">
              {RYTHME.map((s, i) => (
                <span key={s.semaine} className="flex h-full flex-1 flex-col items-center justify-end gap-[5px]">
                  <span className="font-mono text-[10px] text-tx-5">{s.nombre}</span>
                  <span
                    className={cn("w-full max-w-[22px] rounded-t-r2", i === RYTHME.length - 1 ? "bg-ac" : "bg-tint-ac-tx")}
                    style={{ height: `${(s.nombre / MAX_RYTHME) * 100 - 14}%` }}
                  />
                </span>
              ))}
            </span>
            <span className="flex gap-[14px]">
              {RYTHME.map((s) => (
                <span key={s.semaine} className="flex-1 whitespace-nowrap text-center font-mono text-[9.5px] text-tx-5">
                  {s.semaine}
                </span>
              ))}
            </span>
          </Bloc>
        </div>
        <div className="grid gap-3 lg:grid-cols-2">
          <Bloc titre="Taux de réponse par canal">
            {CANAUX.map((c) => (
              <span key={c.libelle} className="grid grid-cols-[minmax(0,1fr)_36px_30px] items-center gap-[10px] text-[12px] sm:grid-cols-[150px_minmax(0,1fr)_36px_30px]">
                <span className="truncate">{c.libelle}</span>
                <span className="hidden h-1 rounded-r2 bg-chip sm:block">
                  <span className="block h-1 rounded-r2 bg-ac" style={{ width: `${c.part}%` }} />
                </span>
                <span className="text-right font-mono text-[10.5px]">{c.part}%</span>
                <span className="text-right font-mono text-[10.5px] text-tx-5">{c.fraction}</span>
              </span>
            ))}
          </Bloc>
          <Bloc titre="Ce que disent ces chiffres" className="hidden lg:flex">
            <span className="flex gap-[9px] text-[12.5px] leading-[1.5]">
              <StatusGlyph ton="g" className="mt-1" />
              Votre réseau répond deux fois et demie plus que les offres en ligne : 67 % contre 25 %.
            </span>
            <span className="flex gap-[9px] text-[12.5px] leading-[1.5]">
              <StatusGlyph ton="a" className="mt-1" />
              Deux candidatures dépassent 14 jours sans réponse : Nord Réseaux et Maison Rivet.
            </span>
          </Bloc>
        </div>
      </div>
    </div>
  );
}
