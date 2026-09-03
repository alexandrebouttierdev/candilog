import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { settingsService } from "../services/settingsService";
import { resetOnboarding } from "@/features/onboarding/model/onboarding-storage";

function errorDetail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Orchestration des sauvegardes, restaurations et remises à zéro locales. */
export function useBackupsViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const setOnboarding = useUiStore((state) => state.setOnboarding);
  const [resetOpen, setResetOpen] = useState(false);
  const [restoreOpen, setRestoreOpen] = useState(false);

  const exportMutation = useMutation({
    mutationFn: settingsService.export,
    onSuccess: (exported) => {
      if (exported) notify({ tone: "success", title: "Sauvegarde créée" });
    },
    onError: (error: unknown) => {
      notify({ tone: "error", title: "Export impossible", detail: errorDetail(error) });
    },
  });
  const restoreMutation = useMutation({
    mutationFn: settingsService.restore,
    onSuccess: async (restored) => {
      setRestoreOpen(false);
      if (!restored) return;
      await queryClient.invalidateQueries();
      notify({ tone: "success", title: "Sauvegarde restaurée" });
    },
    onError: (error: unknown) => {
      notify({ tone: "error", title: "Restauration impossible", detail: errorDetail(error) });
    },
  });
  const resetMutation = useMutation({
    mutationFn: settingsService.reset,
    onSuccess: async (outcome) => {
      await queryClient.invalidateQueries();
      if (!outcome.data_cleared) {
        notify({
          tone: "error",
          title: "Réinitialisation incomplète",
          detail: "Les données locales n’ont pas toutes été supprimées.",
        });
      } else if (!outcome.secret_cleared) {
        notify({
          tone: "error",
          title: "Données effacées, clé encore présente",
          detail: "Supprimez manuellement la clé Candilog dans le coffre de mots de passe du système.",
        });
      } else {
        notify({ tone: "success", title: "Données et clé API réinitialisées" });
      }
      // Une base vidée, c'est une application neuve : la présentation se rejoue comme au
      // premier lancement. Seulement si les données sont réellement parties.
      if (outcome.data_cleared) {
        resetOnboarding();
        setOnboarding(true);
      }
      setResetOpen(false);
    },
    onError: (error: unknown) => {
      notify({ tone: "error", title: "Réinitialisation impossible", detail: errorDetail(error) });
    },
  });

  const busy = exportMutation.isPending
    ? "export"
    : restoreMutation.isPending
      ? "import"
      : resetMutation.isPending
        ? "reset"
        : null;

  return {
    busy,
    resetOpen,
    restoreOpen,
    openReset: () => setResetOpen(true),
    closeReset: () => setResetOpen(false),
    openRestore: () => setRestoreOpen(true),
    closeRestore: () => setRestoreOpen(false),
    exportBackup: exportMutation.mutateAsync,
    restoreBackup: restoreMutation.mutateAsync,
    resetData: resetMutation.mutateAsync,
  };
}
