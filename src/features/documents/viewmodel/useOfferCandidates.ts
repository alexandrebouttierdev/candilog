import { useQuery } from "@tanstack/react-query";
import { APPLICATIONS_KEY, EMPTY_FILTER, applicationService } from "@/features/applications";
import type { Application } from "@/features/applications";

/** Candidatures proposées comme offre visée : les plus récentes encore ouvertes. */
export function useOfferCandidates(enabled: boolean) {
  return useQuery({
    queryKey: [...APPLICATIONS_KEY, "offres-visees"],
    queryFn: () =>
      applicationService.listPage({
        page: 1,
        page_size: 12,
        filter: {
          ...EMPTY_FILTER,
          status: ["EN_ATTENTE", "RELANCEE", "ENTRETIEN"],
          search: "",
          sort: "date",
          descending: true,
          ids: [],
        },
      }),
    enabled,
  });
}

/**
 * Texte d'offre tiré d'une candidature : tout ce que Candilog en sait, sans rien inventer.
 * Le lien de l'offre y figure, mais le modèle ne l'ouvre pas — d'où l'invitation, à l'écran,
 * à coller le texte complet de l'annonce pour un ciblage plus fin.
 */
export function offerTextOf(application: Application): string {
  const lines = [
    `${application.job_title} — ${application.company_name ?? "Entreprise non renseignée"}`,
    [
      application.contract_type_name ?? application.contract_type_code,
      application.effective_city,
      application.professional_domain_name,
    ]
      .filter(Boolean)
      .join(" · "),
    application.notes ?? "",
    application.job_url ? `Annonce : ${application.job_url}` : "",
  ];
  return lines.filter((line) => line.trim() !== "").join("\n");
}
