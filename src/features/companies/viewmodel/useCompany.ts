import { useCallback } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { companyService } from "../services/companyService";
import type { Company, NewCompany } from "@/shared/types/generated/companies";
import { COMPANIES_KEY } from "./companyKeys";
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
 * Page du répertoire pour un sélecteur d'entreprise.
 *
 * La recherche, le tri et la pagination restent en base : le sélecteur ne reçoit qu'une
 * page, déjà réduite au terme saisi. La vue n'appelle donc pas le service elle-même
 * (`docs/CODE_RULES.md` §4) et n'a pas à connaître la forme du filtre côté Rust.
 */
export function useCompanySearch() {
  return useCallback(
    async ({
      page,
      page_size,
      search,
    }: {
      page: number;
      page_size: number;
      search: string;
    }) => {
      const result = await companyService.listPage({
        page,
        page_size,
        filter: {
          search,
          sector_id: null,
          company_type_id: null,
          company_size: null,
          relation_state: null,
        },
      });
      return {
        ...result,
        items: result.items.map((company) => ({
          id: company.id,
          label: company.name,
          meta: [company.sector_name, company.city].filter(Boolean).join(" · ") || undefined,
        })),
      };
    },
    [],
  );
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
