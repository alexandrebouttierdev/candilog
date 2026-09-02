import { Icon } from "@/components/ui/Icon";

import { Champ, EtiquetteMono } from "./primitives";

/** Écran 04 — modale « Nouvelle candidature », telle que `ApplicationFormModal` la rend.
 *
 *  Quatre sections, 680px de large, et la note de pied sur le format des dates. Il n'y a
 *  pas de pièces jointes : un CV n'est pas « joint » à une candidature, il vit dans la
 *  bibliothèque de documents. */
export function EcranCandidature() {
  return (
    <div className="grid min-h-[320px] place-items-center bg-surface-alt p-[22px]">
      <div className="w-full max-w-[680px] overflow-hidden rounded-panel border border-control bg-surface">
        {/* ── En-tête ─────────────────────────────────────────────────────── */}
        <div className="flex items-start gap-[10px] border-b border-line-soft px-[18px] py-[14px]">
          <span className="mt-px text-ink-faint">
            <Icon name="work" size={19} />
          </span>
          <div className="min-w-0 flex-1">
            <p className="text-[14.5px] font-semibold text-ink">Nouvelle candidature</p>
            <p className="mt-[3px] truncate text-[11.5px] text-ink-faint">
              Renseignez le poste et l&apos;entreprise visés
            </p>
          </div>
          <span className="text-ink-faint">
            <Icon name="close" size={17} />
          </span>
        </div>

        <div className="flex flex-col gap-5 px-[18px] py-4">
          {/* ── Poste visé ────────────────────────────────────────────────── */}
          <div className="flex flex-col gap-3">
            <EtiquetteMono>Poste visé</EtiquetteMono>
            <Champ libelle="Poste" obligatoire>
              Designer produit
            </Champ>
            <Champ libelle="Entreprise" obligatoire>
              <span className="grid size-4 place-items-center rounded-[5px] bg-tint-12 text-[9px] font-semibold text-accent-text">
                AN
              </span>
              Atelier Nord
              <span className="ml-auto text-ink-tertiary">
                <Icon name="expand_more" size={15} />
              </span>
            </Champ>
            <div className="grid grid-cols-[repeat(auto-fit,minmax(min(180px,100%),1fr))] gap-3">
              <Champ libelle="Domaine professionnel">
                Communication / Multimédia
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="expand_more" size={15} />
                </span>
              </Champ>
              <Champ libelle="Type de candidature">
                Offre
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="expand_more" size={15} />
                </span>
              </Champ>
            </div>
          </div>

          {/* ── Contrat ───────────────────────────────────────────────────── */}
          <div className="flex flex-col gap-3">
            <EtiquetteMono>Contrat</EtiquetteMono>
            <div className="grid grid-cols-[repeat(auto-fit,minmax(min(140px,100%),1fr))] gap-3">
              <Champ libelle="Type de contrat" obligatoire>
                CDI
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="expand_more" size={15} />
                </span>
              </Champ>
              <Champ libelle="Durée hebdomadaire">
                Temps plein
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="expand_more" size={15} />
                </span>
              </Champ>
              <Champ libelle="Heures par semaine" aide="heures / semaine" mono>
                35
              </Champ>
            </div>
          </div>

          {/* ── Surcharges : ce qui n'est pas saisi est hérité ─────────────── */}
          <div className="flex flex-col gap-3">
            <EtiquetteMono>Informations propres à cette candidature</EtiquetteMono>
            <div className="grid grid-cols-[repeat(auto-fit,minmax(min(180px,100%),1fr))] gap-3">
              <Champ libelle="Ville" aide="Ville de l'entreprise : Lyon" attenue>
                Lyon
              </Champ>
              <Champ libelle="Type d'entreprise" aide="Type de l'entreprise : Startup" attenue>
                Hériter — Startup
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="expand_more" size={15} />
                </span>
              </Champ>
            </div>
          </div>

          {/* ── Suivi ─────────────────────────────────────────────────────── */}
          <div className="flex flex-col gap-3">
            <EtiquetteMono>Suivi</EtiquetteMono>
            <div className="grid grid-cols-[repeat(auto-fit,minmax(min(140px,100%),1fr))] gap-3">
              <Champ libelle="Statut">
                En attente
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="expand_more" size={15} />
                </span>
              </Champ>
              <Champ libelle="Date d'envoi" obligatoire focus mono>
                02-08-2026
                <span className="ml-auto text-ink-tertiary">
                  <Icon name="calendar_month" size={15} />
                </span>
              </Champ>
            </div>
            <Champ libelle="Lien de l'offre" obligatoire>
              <span className="truncate text-ink-muted">
                https://emplois.example/offre/4821
              </span>
            </Champ>
          </div>
        </div>

        {/* ── Pied ─────────────────────────────────────────────────────────── */}
        <div className="flex flex-wrap items-center gap-2 border-t border-line-soft bg-surface-alt px-[18px] py-3">
          <span className="text-[11px] text-ink-tertiary">
            Les dates sont saisies au format JJ-MM-AAAA.
          </span>
          <span className="ml-auto flex gap-2">
            <span className="inline-flex h-[30px] items-center rounded-control border border-control bg-surface px-3 text-[12.5px] font-semibold text-ink">
              Annuler
            </span>
            <span className="inline-flex h-[30px] items-center gap-[6px] rounded-control border border-accent-strong bg-accent px-3 text-[12.5px] font-semibold text-on-accent">
              <Icon name="check" size={15} />
              Enregistrer
            </span>
          </span>
        </div>
      </div>
    </div>
  );
}
