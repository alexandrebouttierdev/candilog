import { useQuery } from "@tanstack/react-query";
import { referentialService } from "../services/referentialService";
import type { ReferenceItem, Referentials } from "../services/referentialService";

/** Clé de cache des référentiels. */
export const REFERENTIALS_KEY = ["referentiels"] as const;

/** Référentiels vides, servis tant que la requête n'a pas abouti. */
const VIDES: Referentials = {
  sectors: [],
  professional_domains: [],
  company_types: [],
  contract_types: [],
};

/**
 * Charge les quatre référentiels pour les sélecteurs et les filtres.
 *
 * `staleTime: Infinity` : les listes sont figées par le schéma et ne changent pas pendant
 * la session. Les recharger à chaque ouverture de formulaire serait un aller-retour IPC
 * pour un résultat identique.
 */
export function useReferentials() {
  const query = useQuery({
    queryKey: REFERENTIALS_KEY,
    queryFn: referentialService.load,
    staleTime: Number.POSITIVE_INFINITY,
  });

  return { ...query, data: query.data ?? VIDES };
}

/**
 * Libellé français associé à un code, ou `null` si le code est absent du référentiel.
 *
 * L'interface n'affiche jamais un code brut : `M18` ou `IT_SERVICES_COMPANY` ne veulent
 * rien dire pour l'utilisateur, qui a choisi « Informatique / Télécommunication ».
 */
export function referenceLabel(
  items: readonly ReferenceItem[],
  code: string | null | undefined,
): string | null {
  if (!code) return null;
  return items.find((item) => item.code === code)?.name ?? null;
}
