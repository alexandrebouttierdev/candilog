import { Icon } from "@/components/ui/Icon";

import { Chip, EcranDeuxVolets, EtiquetteMono } from "./primitives";

const COMPETENCES = [
  { libelle: "système de composants", accentuee: true },
  { libelle: "recherche utilisateur", accentuee: true },
  { libelle: "revues de design", accentuee: false },
  { libelle: "binôme technique", accentuee: false },
  { libelle: "accessibilité", accentuee: false },
  { libelle: "prototypage", accentuee: false },
] as const;

const PROFIL = [
  { couverte: true, libelle: "Système de composants — 3 expériences" },
  { couverte: true, libelle: "Prototypage — mentionné" },
  { couverte: false, libelle: "Recherche utilisateur — à formuler" },
  { couverte: false, libelle: "Accessibilité — absente du CV" },
] as const;

/** Écran 02 — Analyse de l'offre : compétences citées, couverture du profil,
 *  notes de lecture et proposition à valider. */
export function EcranAnalyse() {
  return (
    <EcranDeuxVolets
      gauche={
        <>
          <EtiquetteMono>Compétences citées dans l&apos;offre</EtiquetteMono>
          <div className="mt-3 flex flex-wrap gap-[6px]">
            {COMPETENCES.map(({ libelle, accentuee }) => (
              <Chip key={libelle} accentuee={accentuee}>
                {libelle}
              </Chip>
            ))}
          </div>

          <div className="mt-[18px]">
            <EtiquetteMono>Dans votre profil</EtiquetteMono>
          </div>
          <div className="mt-[10px] border-t border-line">
            {PROFIL.map(({ couverte, libelle }) => (
              <div key={libelle} className="flex items-center gap-[9px] border-b border-line py-2">
                <span className={couverte ? "text-success" : "text-warning"}>
                  <Icon name={couverte ? "check_circle" : "error"} size={15} />
                </span>
                <span className="text-[12.5px] text-ink">{libelle}</span>
              </div>
            ))}
          </div>
        </>
      }
      droite={
        <>
          <EtiquetteMono>Notes de lecture</EtiquetteMono>
          <p className="mt-3 text-[12.5px] leading-[1.7] text-ink-muted">
            L&apos;annonce insiste sur l&apos;autonomie de bout en bout et sur l&apos;animation des
            revues. Deux points à faire apparaître explicitement dans le CV et dans la lettre.
          </p>

          <div className="mt-4 rounded-tile border border-line bg-surface p-3">
            <p className="text-[12.5px] font-semibold text-ink">Proposition à valider</p>
            <p className="mt-[6px] text-[12px] leading-[1.6] text-ink-muted">
              Reformuler l&apos;expérience « refonte du parcours d&apos;inscription » en mentionnant
              les entretiens utilisateurs menés.
            </p>
            <div className="mt-[10px] flex gap-[6px]">
              <span className="inline-flex h-[26px] items-center rounded-control border border-accent-strong bg-accent px-[10px] text-[12px] font-semibold text-on-accent">
                Appliquer
              </span>
              <span className="inline-flex h-[26px] items-center rounded-control border border-control bg-surface px-[10px] text-[12px] font-semibold text-ink">
                Ignorer
              </span>
            </div>
          </div>
        </>
      }
    />
  );
}
