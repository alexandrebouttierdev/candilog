import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { cn } from "@/shared/lib/cn";
import { Button, IconButton } from "@/shared/ui";
import { iaEstConfiguree } from "@/features/settings/model/etatIa";
import { idProvider } from "@/features/settings/model/providers";
import { settingsService } from "@/features/settings/services/settingsService";
import {
  managedOllamaService,
  MANAGED_OLLAMA_KEY,
} from "@/features/settings/services/managedOllamaService";
import { SETTINGS_KEY } from "@/features/settings/viewmodel/useSettingsViewModel";
import { AiBenchmarkModal } from "@/features/ai/view/components/AiBenchmarkModal";
import { AiQuickSelector } from "@/features/ai/view/components/AiQuickSelector";

/** En-tête IA global : sélecteur rapide, benchmark et accès aux réglages. */
export function AiGlobalHeader({ shellBrand = false }: { shellBrand?: boolean }) {
  const navigate = useNavigate();
  const [benchmarkOpen, setBenchmarkOpen] = useState(false);

  const settings = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: settingsService.load,
  });
  const managed = useQuery({
    queryKey: MANAGED_OLLAMA_KEY,
    queryFn: managedOllamaService.status,
  });

  const llm = settings.data?.llm;
  const providerId = llm ? idProvider(llm.provider) : null;
  const configured = llm ? iaEstConfiguree(llm) : false;

  const modelLabel =
    providerId === "candilog_local"
      ? (managed.data?.active_model?.display_name ?? "IA locale")
      : llm?.model.trim() || "modèle";

  const canBenchmark =
    providerId === "candilog_local"
      ? Boolean(managed.data?.active_model)
      : configured && Boolean(llm?.model.trim());

  return (
    <>
      <AiQuickSelector shellBrand={shellBrand} />
      <Button
        variant="secondary"
        icon="bolt"
        disabled={!canBenchmark}
        className={cn(
          shellBrand &&
            "shell-brand-control disabled:border-white/15 disabled:bg-white/5 disabled:text-white/40",
        )}
        onClick={() => setBenchmarkOpen(true)}
      >
        Tester
      </Button>
      <IconButton
        icon="settings"
        label="Réglages Intelligence artificielle"
        className={cn(
          shellBrand &&
            "shell-brand-control disabled:border-white/15 disabled:bg-white/5 disabled:text-white/40",
        )}
        onClick={() => {
          void navigate("/settings/ai");
        }}
      />
      <AiBenchmarkModal
        open={benchmarkOpen}
        onClose={() => setBenchmarkOpen(false)}
        modelLabel={modelLabel}
      />
    </>
  );
}
