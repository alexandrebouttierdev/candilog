import { Icon } from "@/components/ui/Icon";
import { cn } from "@/lib/cn";

import { EtiquetteMono, Surlignage } from "./primitives";

const VERSIONS = [
  { libelle: "CV — Atelier Nord", date: "02-08-2026", score: 84, active: true },
  { libelle: "CV — Cobalt Bureau", date: "21-08-2026", score: 78, active: false },
  { libelle: "CV — base", date: "18-07-2026", score: null, active: false },
] as const;

/** Vignette de document : la miniature à traits de `ResumeLibraryPage`. */
function Vignette({ active }: { active: boolean }) {
  return (
    <span className="flex h-[50px] w-[38px] flex-none flex-col gap-[3px] rounded-[6px] border border-line bg-page px-[5px] py-[6px]">
      <span className={cn("h-[3px] w-[70%] rounded-sm", active ? "bg-accent" : "bg-tint-12")} />
      <span className="h-[2px] w-full rounded-sm bg-line" />
      <span className="h-[2px] w-[85%] rounded-sm bg-line" />
      <span className="h-[2px] w-[95%] rounded-sm bg-line" />
      <span className="h-[2px] w-[60%] rounded-sm bg-line" />
    </span>
  );
}

/** Écran 03 — Documents → Mes CV : bibliothèque à gauche, aperçu A4 à droite.
 *
 *  La liste est une vraie bibliothèque paginée, avec sa vignette, sa date et son score ATS
 *  sur la version sélectionnée — pas une simple liste de fichiers. */
export function EcranDocuments() {
  return (
    <div className="grid min-h-[320px] grid-cols-[repeat(auto-fit,minmax(min(280px,100%),1fr))]">
      {/* ── Bibliothèque ─────────────────────────────────────────────────── */}
      <div className="min-w-0 border-r border-line-soft">
        <div className="border-b border-line-soft px-[18px] pt-4 pb-3">
          <div className="mb-[11px] flex items-center justify-between">
            <span className="text-[13px] font-semibold text-ink">Bibliothèque</span>
            <span className="text-[11.5px] text-ink-faint">3 versions</span>
          </div>
          <div className="flex h-8 items-center gap-2 rounded-control border border-line bg-page px-[10px] text-[12px] text-ink-faint">
            <Icon name="search" size={16} />
            Rechercher une version…
          </div>
        </div>

        <div className="flex flex-col gap-[6px] p-[10px]">
          {VERSIONS.map((version) => (
            <div
              key={version.libelle}
              className={cn(
                "flex gap-3 rounded-tile border p-3",
                version.active ? "border-tint-border bg-tint-08" : "border-transparent",
              )}
            >
              <Vignette active={version.active} />
              <span className="min-w-0 flex-1">
                <span className="mb-[3px] flex items-center gap-2">
                  <span className="truncate text-[12.5px] font-semibold text-ink">
                    {version.libelle}
                  </span>
                  {version.active && version.score !== null ? (
                    <span className="inline-flex h-[18px] flex-none items-center rounded-pill border border-success-border bg-success-tint px-[6px] text-[10.5px] font-semibold tabular-nums text-success-text">
                      ATS {version.score}
                    </span>
                  ) : null}
                </span>
                <span className="block text-[11px] tabular-nums text-ink-tertiary">
                  {version.date}
                </span>
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* ── Aperçu ───────────────────────────────────────────────────────── */}
      <div className="min-w-0 bg-surface-alt">
        <div className="flex items-center justify-between gap-3 border-b border-line-soft px-[18px] py-3">
          <div className="flex min-w-0 items-center gap-[9px]">
            <span className="text-ink-faint">
              <Icon name="visibility" size={16} />
            </span>
            <p className="truncate text-[12.5px] text-ink">Aperçu · CV — Atelier Nord</p>
          </div>
          <div className="flex flex-none items-center gap-[6px] text-ink-faint">
            <Icon name="edit" size={16} />
            <Icon name="content_copy" size={16} />
            <Icon name="picture_as_pdf" size={16} />
          </div>
        </div>

        <div className="px-[18px] py-4">
          <div className="rounded-tile border border-line bg-surface px-[18px] py-4">
            <p className="text-[13px] font-semibold text-ink">Camille Berthier</p>
            <p className="mt-[2px] text-[11.5px] text-ink-tertiary">Designer produit · Lyon</p>
            <div className="my-3 h-px bg-line-soft" />

            <EtiquetteMono className="text-[10px]">Expérience</EtiquetteMono>
            <p className="mt-2 text-[11.5px] leading-[1.7] text-ink-muted">
              Refonte du parcours d&apos;inscription, de la{" "}
              <Surlignage>recherche utilisateur</Surlignage> à la mise en production. Animation
              des <Surlignage>revues de design</Surlignage> hebdomadaires.
            </p>

            <EtiquetteMono className="mt-3 text-[10px]">Compétences</EtiquetteMono>
            <p className="mt-2 text-[11.5px] leading-[1.7] text-ink-muted">
              Système de composants · Prototypage · <Surlignage>Accessibilité</Surlignage>
            </p>
          </div>
          <p className="mt-[10px] font-mono text-[10.5px] text-ink-faint">
            Version enregistrée avec son score et sa date
          </p>
        </div>
      </div>
    </div>
  );
}
