import { useQuery } from "@tanstack/react-query";
import { documentsService } from "../services/documentsService";
import { COVER_LETTERS_KEY, RESUME_KEY } from "./documentKeys";
import type { DocumentFilter } from "./useDocumentsViewModel";

/**
 * Décomptes des onglets de la barre d'outils de Documents.
 *
 * Même rôle que `useNavCounts` pour la navigation, et **mêmes clés de cache** pour les CV et
 * les lettres : le décompte de l'onglet et celui de la navigation partagent donc une seule
 * requête, et une écriture qui invalide la racine de la feature les met à jour ensemble.
 *
 * Ces requêtes vivent dans le ViewModel et non dans la page : la vue ne parle pas au service
 * (`docs/CODE_RULES.md` §4), et la logique reste testable sans monter l'écran.
 */
export function useDocumentCounts(): Record<DocumentFilter, number | undefined> {
  const resumes = useQuery({
    queryKey: [...RESUME_KEY, "navigation"],
    queryFn: () => documentsService.listResumePage({ page: 1, page_size: 1, search: "" }),
  });
  const letters = useQuery({
    queryKey: [...COVER_LETTERS_KEY, "navigation"],
    queryFn: () => documentsService.listCoverLettersPage({ page: 1, page_size: 1, search: "" }),
  });
  const analyses = useQuery({
    queryKey: [...RESUME_KEY, "navigation", "analyses"],
    queryFn: () =>
      documentsService.listResumePage({
        page: 1,
        page_size: 1,
        search: "",
        scored_only: true,
      }),
  });

  return {
    // Un décompte partiel n'est pas affiché : additionner une moitié chargée afficherait un
    // total faux le temps de l'autre requête.
    all:
      resumes.data && letters.data ? resumes.data.total + letters.data.total : undefined,
    resumes: resumes.data?.total,
    letters: letters.data?.total,
    analyses: analyses.data?.total,
  };
}
