import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMutation, useQuery } from "@tanstack/react-query";
import { AppError } from "@/shared/types/app-error";
import type { UpdateInfo, UpdateProgress } from "@/shared/types/generated/settings";
import { settingsService } from "../services/settingsService";
import { A_ABOUT_KEY } from "./useSettingsViewModel";

const RELEASES_PAGE = "https://github.com/alexandrebouttierdev/candilog/releases/latest";

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof AppError ? error.message : fallback;
}

function officialReleasePage(url: string): string {
  try {
    const parsed = new URL(url);
    if (
      parsed.protocol === "https:" &&
      parsed.hostname === "github.com" &&
      parsed.pathname.startsWith("/alexandrebouttierdev/candilog/")
    ) {
      return parsed.toString();
    }
  } catch {
    // La page de repli officielle est utilisée ci-dessous.
  }
  return RELEASES_PAGE;
}

/** Version installée, vérification et téléchargement des mises à jour. */
export function useUpdatesViewModel() {
  const [update, setUpdate] = useState<UpdateInfo | null | undefined>(undefined);
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const about = useQuery({ queryKey: A_ABOUT_KEY, queryFn: settingsService.about });

  useEffect(() => {
    let cancelled = false;
    let dispose: (() => void) | undefined;
    void listen<UpdateProgress>("update-progress", (event) => {
      if (!cancelled) setProgress(event.payload.progress);
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else dispose = unlisten;
      })
      .catch(() => {
        // La revue dans un navigateur ne dispose pas du runtime Tauri.
      });
    return () => {
      cancelled = true;
      dispose?.();
    };
  }, []);

  const checkMutation = useMutation({
    mutationFn: settingsService.checkUpdate,
    onMutate: () => setError(null),
    onSuccess: setUpdate,
    onError: (caught: unknown) => setError(errorMessage(caught, "Vérification impossible.")),
  });
  const downloadMutation = useMutation({
    mutationFn: settingsService.downloadUpdate,
    onMutate: () => {
      setError(null);
      setProgress(0);
    },
    onError: (caught: unknown) => setError(errorMessage(caught, "Téléchargement impossible.")),
  });

  // Ni `check` ni `download` ne rejettent : leur échec est déjà porté par `error`, que
  // l'écran affiche. Propager en plus un rejet obligeait chaque appelant à l'attraper pour
  // ne rien en faire, et un `void vm.check()` laissait filer un rejet non traité.
  async function check(): Promise<void> {
    await checkMutation.mutateAsync().catch(() => undefined);
  }

  async function download(): Promise<void> {
    if (!update?.asset) {
      await settingsService
        .openReleasePage(officialReleasePage(update?.page_url ?? RELEASES_PAGE))
        .catch(() => setError("Page des versions inaccessible."));
      return;
    }
    await downloadMutation.mutateAsync().catch(() => undefined);
  }

  // Typé explicitement : la déduction élargirait le ternaire à `string`, et l'écran ne
  // pourrait plus distinguer une vérification d'un téléchargement.
  const busy: "check" | "download" | null = checkMutation.isPending
    ? "check"
    : downloadMutation.isPending
      ? "download"
      : null;

  return {
    version: about.data?.version ?? (about.isError ? "inconnue" : "…"),
    update,
    busy,
    progress,
    error,
    check,
    download,
  };
}
