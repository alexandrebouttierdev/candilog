import { useEffect } from "react";
import { AppProviders } from "./providers/AppProviders";
import { AppRouter } from "./router/AppRouter";
import { Toaster } from "@/shared/ui";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import { settingsService } from "@/features/settings/services/settingsService";
import { OnboardingTour } from "@/features/onboarding/view/components/OnboardingTour";
import { markOnboardingCompleted, onboardingCompleted } from "@/features/onboarding/model/onboarding-storage";

export function App() {
  const theme = useUiStore((state) => state.theme);
  // L'affichage du tour vit dans le store : les Réglages doivent pouvoir le rouvrir après
  // une réinitialisation des données, depuis un autre écran.
  const onboarding = useUiStore((state) => state.onboarding);
  const setOnboarding = useUiStore((state) => state.setOnboarding);

  useEffect(() => {
    if (!onboardingCompleted()) setOnboarding(true);
  }, [setOnboarding]);

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
      {onboarding ? (
        <OnboardingTour
          onFinish={() => {
            markOnboardingCompleted();
            setOnboarding(false);
          }}
        />
      ) : null}
    </AppProviders>
  );
}
