import type { ReactNode } from "react";
import { useState } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { createQueryClient } from "./query-client";
import { useBootstrapTheme } from "@/features/settings";

/** Charge le thème persisté une fois le client de requêtes monté. */
function BootstrapTheme() {
  useBootstrapTheme();
  return null;
}

/** Fournisseurs transverses montés une seule fois autour de l'application. */
export function AppProviders({ children }: { children: ReactNode }) {
  // `useState` et non une constante de module : un client créé à l'import serait partagé
  // entre les tests, qui hériteraient alors du cache les uns des autres.
  const [queryClient] = useState(createQueryClient);

  return (
    <QueryClientProvider client={queryClient}>
      <BootstrapTheme />
      {children}
    </QueryClientProvider>
  );
}
