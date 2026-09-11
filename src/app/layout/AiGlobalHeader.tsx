import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/shared/lib/cn";
import { Button, IconButton } from "@/shared/ui";
import {
  idProvider,
  isAiConfigured,
  useManagedOllamaViewModel,
  useSettingsViewModel,
} from "@/features/settings";
import { AiBenchmarkModal, AiQuickSelector } from "@/features/ai";

/** En-tête IA global : sélecteur rapide, benchmark et accès aux réglages. */
export function AiGlobalHeader({ shellBrand = false }: { shellBrand?: boolean }) {
  const navigate = useNavigate();
  const [benchmarkOpen, setBenchmarkOpen] = useState(false);

  const settings = useSettingsViewModel();
  const managed = useManagedOllamaViewModel(undefined, { enabled: true });

  const llm = settings.data?.llm;
  const providerId = llm ? idProvider(llm.provider) : null;
  const configured = llm ? isAiConfigured(llm) : false;

  const modelLabel =
    providerId === "candilog_local"
      ? (managed.status?.active_model?.display_name ?? "IA locale")
      : llm?.model.trim() || "modèle";

  const canBenchmark =
    providerId === "candilog_local"
      ? Boolean(managed.status?.active_model)
      : configured && Boolean(llm?.model.trim());

  return (
    <>
      <AiQuickSelector shellBrand={shellBrand} />
      <Button
        variant="secondary"
        icon="bolt"
        disabled={!canBenchmark}
        className={cn(shellBrand && "shell-brand-control")}
        onClick={() => setBenchmarkOpen(true)}
      >
        Tester
      </Button>
      <IconButton
        icon="settings"
        label="Réglages Intelligence artificielle"
        className={cn(shellBrand && "shell-brand-control")}
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
