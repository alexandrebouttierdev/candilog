import { useEffect } from "react";
import { AppProviders } from "./providers/AppProviders";
import { AppRouter } from "./router/AppRouter";
import { Toaster } from "@/shared/ui";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import { settingsService } from "@/features/settings/services/settingsService";

export function App() {
  const theme = useUiStore((state) => state.theme);

  useEffect(() => {
    void settingsService
      .load()
      .then((settings) => {
        useUiStore.getState().setTheme(settings.theme);
      })
      .catch(() => {
        /* Première ouverture : le thème système reste. */
      });
  }, []);

  // Le thème vit sur `document.documentElement`, hors de l'arbre React : un effet est le
  // seul moyen de l'y refléter. `system` retire l'attribut, laissant jouer la préférence
  // du système d'exploitation.
  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  return (
    <AppProviders>
      <AppRouter />
      <Toaster />
    </AppProviders>
  );
}
