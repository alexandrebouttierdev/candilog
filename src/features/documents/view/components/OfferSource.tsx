import { useState } from "react";
import type { Application } from "@/features/applications";
import { formatReference } from "@/features/applications";
import { Avatar, Skeleton } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { offerTextOf, useOfferCandidates } from "../../viewmodel/useOfferCandidates";
import { ChampOffre } from "../pages/documentPageSupport";

/**
 * « Offre visée » des générateurs : une candidature du suivi, ou le texte collé de
 * l'annonce. Choisir une candidature remplit le texte avec ce que Candilog en sait ; il
 * reste modifiable, et le texte complet de l’annonce donne un meilleur ciblage.
 */
export function OfferSource({
  value,
  onChange,
  readClipboard,
  disabled = false,
  textLabel = "Texte de l’offre",
  required = true,
  onPick,
}: {
  value: string;
  onChange: (value: string) => void;
  readClipboard: () => Promise<string>;
  disabled?: boolean;
  /** Libellé du champ en mode « Texte collé ». */
  textLabel?: string;
  /** Le générateur de CV exige une offre ; la lettre peut s'en passer. */
  required?: boolean;
  /** Candidature choisie : la lettre en reprend l'entreprise et le poste. */
  onPick?: (application: Application) => void;
}) {
  const [mode, setMode] = useState<"application" | "text">("text");
  const [picked, setPicked] = useState<string | null>(null);
  const candidates = useOfferCandidates(mode === "application");

  const pick = (application: Application) => {
    setPicked(application.id);
    onChange(offerTextOf(application));
    onPick?.(application);
  };

  return (
    <div>
      <div role="tablist" aria-label="Source de l’offre" className="mb-2.5 flex gap-px rounded-r7 bg-chip p-0.5">
        {(
          [
            { value: "application", label: "Une candidature" },
            { value: "text", label: "Texte collé" },
          ] as const
        ).map((tab) => (
          <button
            key={tab.value}
            type="button"
            role="tab"
            aria-selected={mode === tab.value}
            disabled={disabled}
            onClick={() => setMode(tab.value)}
            className={cn(
              "h-6 flex-1 rounded-r5 text-small",
              mode === tab.value ? "bg-panel font-medium text-tx" : "text-tx-4 hover:text-tx-2",
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {mode === "application" ? (
        candidates.isPending ? (
          <div role="status" aria-label="Chargement des candidatures" className="flex flex-col gap-2">
            {[1, 2, 3].map((index) => (
              <Skeleton key={index} index={index} className="h-9" />
            ))}
          </div>
        ) : (candidates.data?.items.length ?? 0) === 0 ? (
          <p className="text-sub leading-[1.5] text-tx-5">
            Aucune candidature ouverte. Collez le texte de l’offre à la place.
          </p>
        ) : (
          <ul aria-label="Candidatures ouvertes" className="flex flex-col gap-0.5">
            {(candidates.data?.items ?? []).map((application) => (
              <li key={application.id}>
                <button
                  type="button"
                  disabled={disabled}
                  aria-pressed={picked === application.id}
                  onClick={() => pick(application)}
                  className={cn(
                    "flex w-full items-center gap-2.5 rounded-r7 px-2 py-1.5 text-left",
                    picked === application.id ? "bg-panel shadow-[inset_2px_0_0_var(--ac)]" : "hover:bg-hover",
                  )}
                >
                  <Avatar name={application.company_name ?? application.job_title} kind="company" size={20} />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-ui text-tx-2">{application.job_title}</span>
                    <span className="block truncate text-tiny text-tx-5">
                      {application.company_name ?? "—"} · {formatReference(application.reference_number)}
                    </span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )
      ) : null}

      {mode === "text" || picked ? (
        <div className={mode === "application" ? "mt-3" : ""}>
          <ChampOffre
            label={mode === "application" ? "Texte transmis à l’IA" : textLabel}
            required={required}
            help={
              mode === "application"
                ? "Collez le texte complet de l’annonce pour un ciblage plus fin."
                : "Le texte est envoyé uniquement au fournisseur choisi pour cette tâche."
            }
            rows={mode === "application" ? 7 : 14}
            value={value}
            placeholder="Collez ici l’intitulé, les missions et les compétences recherchées…"
            onChange={onChange}
            readClipboard={readClipboard}
            disabled={disabled}
          />
        </div>
      ) : null}
    </div>
  );
}
