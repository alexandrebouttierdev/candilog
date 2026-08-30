import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { Referentials } from "@/features/referentials";

/**
 * Client de test isolé.
 *
 * `retry: false` : sans cela, une erreur simulée serait réessayée et le test attendrait les
 * temporisations de TanStack Query. Un client neuf par appel évite qu'un test hérite du
 * cache du précédent.
 */
export function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
}

/** Enveloppe les composants et hooks qui interrogent TanStack Query. */
export function QueryWrapper({ children }: { children: ReactNode }) {
  return <QueryClientProvider client={createTestQueryClient()}>{children}</QueryClientProvider>;
}

/**
 * Référentiels réduits, pour les composants qui affichent des sélecteurs.
 *
 * Quelques entrées par catalogue suffisent : la complétude des listes est vérifiée côté
 * Rust, sur la base réellement semée.
 */
export const REFERENTIELS_DE_TEST: Referentials = {
  sectors: [
    { id: "5ec70000-0000-4000-8000-00000000000d", name: "Informatique / Télécommunication" },
    { id: "5ec70000-0000-4000-8000-000000000003", name: "Banque / Assurance" },
  ],
  professional_domains: [
    { code: "M18", name: "Informatique / Télécommunication" },
    { code: "C", name: "Banque / Assurance" },
  ],
  company_types: [
    { code: "IT_SERVICES_COMPANY", name: "ESN / Société de services numériques" },
    { code: "FINAL_CLIENT", name: "Client final" },
  ],
  contract_types: [
    { code: "CDI", name: "CDI" },
    { code: "MIS", name: "Intérim" },
  ],
};
