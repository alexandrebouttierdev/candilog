import { useQuery } from "@tanstack/react-query";
// Modules feuilles et non barils de feature : la coque charge ce hook à chaque démarrage, et
// un baril tire tout ce qu'il réexporte. Les écrans lourds (éditeur de CV, feuilles A4)
// atterrissaient ainsi dans le chunk d'entrée, ce qui annulait le `lazy()` d'`AppRouter`.
import { applicationService } from "@/features/applications/services/applicationService";
import { APPLICATIONS_KEY } from "@/features/applications/viewmodel/useApplicationsViewModel";
import { EMPTY_FILTER } from "@/features/applications/model/schemas/application-filter.schema";
import type { ApplicationFilter } from "@/features/applications/services/applicationService";
import { COMPANIES_KEY } from "@/features/companies/viewmodel/companyKeys";
import { companyService } from "@/features/companies/services/companyService";
import { CONTACTS_KEY } from "@/features/contacts/viewmodel/contactKeys";
import { contactService } from "@/features/contacts/services/contactService";
import { COVER_LETTERS_KEY, RESUME_KEY } from "@/features/documents/viewmodel/documentKeys";
import { documentsService } from "@/features/documents/services/documentsService";
import { horizonOf } from "@/features/analytics/model/agenda";
import { useAgenda } from "@/features/analytics/viewmodel/useAgenda";
import { PROFILE_KEY } from "@/features/profile/viewmodel/useProfileViewModel";
import { profileService } from "@/features/profile/services/profileService";

/** Toutes les candidatures, sans filtre : le décompte de la navigation est un total. */
const TOUTES: ApplicationFilter = { ...EMPTY_FILTER, search: "", sort: "date", descending: true, ids: [] };

/**
 * Décomptes de la barre de navigation (`INTERACTIONS.md` §2) : échéances du jour, total
 * des candidatures, entreprises + contacts, documents, complétude du profil.
 *
 * Chaque requête est rangée sous la clé racine de sa feature : une écriture qui invalide
 * les candidatures, les relances ou le profil met donc aussi le décompte à jour. Un
 * décompte inconnu (chargement, erreur) n'est pas affiché plutôt que d'afficher 0.
 */
export function useNavCounts() {
  const applications = useQuery({
    queryKey: [...APPLICATIONS_KEY, "navigation"],
    queryFn: () => applicationService.breakdown(TOUTES),
  });
  const companies = useQuery({
    queryKey: [...COMPANIES_KEY, "navigation"],
    queryFn: () =>
      companyService.listPage({
        page: 1,
        page_size: 1,
        filter: { search: "", sector_id: null, company_type_id: null, company_size: null, relation_state: null },
      }),
  });
  const contacts = useQuery({
    queryKey: [...CONTACTS_KEY, "navigation"],
    queryFn: () => contactService.listPage({ page: 1, page_size: 1, search: "", tracking_role: null }),
  });
  const resumes = useQuery({
    queryKey: [...RESUME_KEY, "navigation"],
    queryFn: () => documentsService.listResumePage({ page: 1, page_size: 1, search: "" }),
  });
  const letters = useQuery({
    queryKey: [...COVER_LETTERS_KEY, "navigation"],
    queryFn: () => documentsService.listCoverLettersPage({ page: 1, page_size: 1, search: "" }),
  });
  const agenda = useAgenda();
  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });

  const breakdown = applications.data;
  return {
    // Ce qui est à traiter aujourd'hui : retards compris, la semaine à venir exclue.
    today: agenda.data?.filter((item) => horizonOf(item, agenda.today) !== "week").length,
    applications: breakdown
      ? breakdown.pending + breakdown.followed_up + breakdown.interview + breakdown.rejected
      : undefined,
    relations:
      companies.data && contacts.data ? companies.data.total + contacts.data.total : undefined,
    documents: resumes.data && letters.data ? resumes.data.total + letters.data.total : undefined,
    profile: profile.data?.completion,
  };
}
