import { Icon } from "@/components/ui/Icon";

import { Champ } from "./primitives";

/** Écran 04 — modale « Nouvelle candidature » centrée sur fond `--surface-alt`.
 *  Le champ « Date d'envoi » porte l'état de focus indigo. */
export function EcranCandidature() {
  return (
    <div className="grid min-h-[320px] place-items-center bg-surface-alt p-[22px]">
      <div className="w-full max-w-[560px] overflow-hidden rounded-panel border border-control bg-surface">
        <div className="flex items-center border-b border-line-soft px-[18px] py-[14px]">
          <p className="text-[14.5px] font-semibold text-ink">Nouvelle candidature</p>
          <span className="ml-auto text-ink-faint">
            <Icon name="close" size={17} />
          </span>
        </div>

        <div className="grid grid-cols-[repeat(auto-fit,minmax(min(200px,100%),1fr))] gap-3 px-[18px] py-4">
          <Champ libelle="Poste" obligatoire>
            Designer produit
          </Champ>

          <Champ libelle="Entreprise" obligatoire>
            <span className="grid size-4 place-items-center rounded-[5px] bg-tint-12 text-[9px] font-semibold text-accent-text">
              AN
            </span>
            Atelier Nord
          </Champ>

          <Champ libelle="Date d'envoi" focus mono>
            02-08-2026
            <span className="ml-auto text-ink-tertiary">
              <Icon name="calendar_month" size={15} />
            </span>
          </Champ>

          <Champ libelle="Statut">
            Envoyée
            <span className="ml-auto text-ink-tertiary">
              <Icon name="expand_more" size={15} />
            </span>
          </Champ>

          <div className="col-span-full">
            <p className="mb-[5px] text-[11.5px] text-ink-muted">Documents joints</p>
            <div className="flex flex-wrap gap-[6px]">
              {[
                { icone: "description", libelle: "CV — Atelier Nord" },
                { icone: "mail", libelle: "Lettre — Atelier Nord" },
              ].map(({ icone, libelle }) => (
                <span
                  key={libelle}
                  className="inline-flex h-[26px] items-center gap-[6px] rounded-[7px] border border-line bg-surface-alt px-[9px] text-[11.5px] text-ink"
                >
                  <span className="text-ink-tertiary">
                    <Icon name={icone} size={14} />
                  </span>
                  {libelle}
                </span>
              ))}
              <span className="inline-flex h-[26px] items-center gap-[6px] rounded-[7px] border border-dashed border-control px-[9px] text-[11.5px] text-ink-tertiary">
                <Icon name="add" size={14} />
                Ajouter
              </span>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 border-t border-line-soft bg-surface-alt px-[18px] py-3">
          <span className="inline-flex h-[30px] items-center rounded-control border border-control bg-surface px-3 text-[12.5px] font-semibold text-ink">
            Annuler
          </span>
          <span className="inline-flex h-[30px] items-center rounded-control border border-accent-strong bg-accent px-3 text-[12.5px] font-semibold text-on-accent">
            Enregistrer
          </span>
        </div>
      </div>
    </div>
  );
}
