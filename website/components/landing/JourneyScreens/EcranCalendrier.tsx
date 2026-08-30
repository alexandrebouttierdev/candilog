import { Icon } from "@/components/ui/Icon";
import { CASES_VIDES_AOUT, MARQUES_CALENDRIER } from "@/lib/data/parcours";

import { EcranDeuxVolets, EtiquetteMono, Pastille } from "./primitives";

const JOURS = ["L", "M", "M", "J", "V", "S", "D"] as const;
const CASES = [
  ...Array.from({ length: CASES_VIDES_AOUT }, () => null),
  ...Array.from({ length: 31 }, (_, i) => i + 1),
];

const A_TRAITER = [
  {
    ton: "accent" as const,
    statut: "Entretien",
    quand: "05 août · 14:30",
    titre: "Atelier Nord — Designer produit",
    detail: "Visio · Claire Nguyen",
  },
  {
    ton: "warning" as const,
    statut: "Relance",
    quand: "08 août",
    titre: "Groupe Vallée — Chargé de projet",
    detail: "Envoyée il y a 11 jours, sans réponse",
  },
];

/** Écran 05 — Calendrier d'août 2026 (4 dates marquées) et cartes à traiter. */
export function EcranCalendrier() {
  return (
    <EcranDeuxVolets
      colonneMin={280}
      gauche={
        <>
          <div className="mb-3 flex items-center gap-[10px]">
            <p className="text-[13px] font-semibold text-ink">Août 2026</p>
            <span className="ml-auto text-ink-faint">
              <Icon name="chevron_left" size={15} />
            </span>
            <span className="text-ink-muted">
              <Icon name="chevron_right" size={15} />
            </span>
          </div>

          <div className="grid grid-cols-7 gap-px overflow-hidden rounded-[9px] border border-line-soft bg-line-soft">
            {JOURS.map((jour, index) => (
              <div
                key={`${jour}-${String(index)}`}
                className="bg-surface-alt py-[5px] text-center font-mono text-[9.5px] text-ink-faint"
              >
                {jour}
              </div>
            ))}
            {CASES.map((numero, index) => {
              const marque = numero === null ? undefined : MARQUES_CALENDRIER[numero];
              return (
                <div
                  key={numero === null ? `vide-${String(index)}` : numero}
                  className="min-h-[38px] bg-surface px-[5px] py-1"
                >
                  <div className="text-[10.5px] tabular-nums text-ink-tertiary">{numero}</div>
                  {marque ? (
                    <div
                      className={`mt-[3px] flex h-[14px] items-center overflow-hidden whitespace-nowrap rounded-[4px] px-[3px] text-[9px] font-semibold ${marque.classes}`}
                    >
                      {marque.libelle}
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </>
      }
      droite={
        <>
          <EtiquetteMono>À traiter cette semaine</EtiquetteMono>
          <div className="mt-3 flex flex-col gap-2">
            {A_TRAITER.map((carte) => (
              <div
                key={carte.titre}
                className="rounded-tile border border-line bg-surface px-3 py-[11px]"
              >
                <div className="flex items-center gap-2">
                  <Pastille ton={carte.ton}>{carte.statut}</Pastille>
                  <span className="ml-auto text-[11.5px] tabular-nums text-ink-tertiary">
                    {carte.quand}
                  </span>
                </div>
                <p className="mt-[7px] text-[12.5px] font-semibold text-ink">{carte.titre}</p>
                <p className="mt-[3px] text-[11.5px] text-ink-tertiary">{carte.detail}</p>
              </div>
            ))}
          </div>
        </>
      }
    />
  );
}
