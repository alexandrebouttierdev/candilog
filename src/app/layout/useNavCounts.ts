import { useQuery } from "@tanstack/react-query";
import { APPLICATIONS_KEY, applicationService } from "@/features/applications";
import type { ApplicationFilter } from "@/features/applications";
import { COMPANIES_KEY, companyService } from "@/features/companies";
import { CONTACTS_KEY, contactService } from "@/features/contacts";
import { COVER_LETTERS_KEY, RESUME_KEY, documentsService } from "@/features/documents";
import { FOLLOW_UPS_KEY, followUpService } from "@/features/followups";
import { interviewService } from "@/features/interviews";
import { PROFILE_KEY, profileService } from "@/features/profile";

/** Toutes les candidatures, sans filtre : le décompte de la navigation est un total. */
const TOUTES: ApplicationFilter = {
  search: "",
  status: [],
  application_type: [],
  channel: [],
  contract_type_code: [],
  professional_domain_id: [],
  company_type_id: [],
  company_size: [],
  sector_id: [],
  weekly_work_schedule: [],
  min_weekly_hours: null,
  max_weekly_hours: null,
  company_id: null,
  city: "",
  job_title: "",
  start_date: null,
  end_date: null,
  sort: "date",
  descending: true,
  ids: [],
};

/** Date locale `AAAA-MM-JJ` : « aujourd'hui » est celui de l'utilisateur, pas l'UTC. */
function localDate(date = new Date()): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/**
 * Décomptes de la barre de navigation (`INTERACTIONS.md` §2) : échéances du jour, total
 * des candidatures, entreprises + contacts, documents, complétude du profil.
 *
 * Chaque requête est rangée sous la clé racine de sa feature : une écriture qui invalide
 * les candidatures, les relances ou le profil met donc aussi le décompte à jour. Un
 * décompte inconnu (chargement, erreur) n'est pas affiché plutôt que d'afficher 0.
 */
export function useNavCounts() {
  const today = localDate();

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
        filter: { search: "", sector_id: null, company_type_id: null, company_size: null },
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
  const followUps = useQuery({
    queryKey: [...FOLLOW_UPS_KEY, "navigation", today],
    queryFn: () => followUpService.listBetween(today, today),
  });
  const interviews = useQuery({
    queryKey: ["entretiens", "navigation", today],
    // Les entretiens portent une heure : les bornes couvrent la journée entière.
    queryFn: () => interviewService.listBetween(`${today}T00:00:00`, `${today}T23:59:59`),
  });
  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });

  const breakdown = applications.data;
  return {
    today:
      followUps.data && interviews.data ? followUps.data.length + interviews.data.length : undefined,
    applications: breakdown
      ? breakdown.pending + breakdown.followed_up + breakdown.interview + breakdown.rejected
      : undefined,
    relations:
      companies.data && contacts.data ? companies.data.total + contacts.data.total : undefined,
    documents: resumes.data && letters.data ? resumes.data.total + letters.data.total : undefined,
    profile: profile.data?.completion,
  };
}
