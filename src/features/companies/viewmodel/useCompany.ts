import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { companyService } from "../services/companyService";
import type { Company, NewCompany } from "../services/companyService";
import { COMPANIES_KEY } from "./useCompaniesViewModel";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/**
 * Fiche d'une entreprise chargée par son identifiant.
 *
 * Sert partout où l'on connaît l'identifiant sans avoir la fiche sous la main : le libellé
 * d'un sélecteur, les valeurs héritées par une candidature. Sortir cette requête du
 * ViewModel complet évite de charger le répertoire, les candidatures liées et les
 * compteurs pour afficher un seul nom.
 */
export function useCompany(id: string | null) {
  return useQuery({
    queryKey: [...COMPANIES_KEY, "detail", id],
    queryFn: () => companyService.get(id as string),
    enabled: id !== null,
  });
}

/**
 * Création d'une entreprise, isolée du ViewModel du répertoire.
 *
 * Même commande, même validation Rust et même invalidation de cache que la création depuis
 * l'écran Relations : seule l'origine du geste change. La fiche créée est déposée dans le
 * cache de détail pour que son libellé soit immédiatement disponible au sélecteur qui vient
 * de la demander, sans aller-retour supplémentaire.
 */
export function useCreateCompany() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  return useMutation({
    mutationFn: (input: NewCompany) => companyService.create(input),
    onSuccess: async (company: Company) => {
      queryClient.setQueryData([...COMPANIES_KEY, "detail", company.id], company);
      await queryClient.invalidateQueries({ queryKey: COMPANIES_KEY });
      notify({ tone: "success", title: "Entreprise enregistrée", detail: company.name });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Enregistrement impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });
}
