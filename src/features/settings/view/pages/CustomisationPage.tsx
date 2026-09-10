import { useState } from "react";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import type { Settings, ThemePref } from "@/shared/types/generated/settings";
import { Button, ErrorBanner, PageHeader, SegmentedControl, Skeleton } from "@/shared/ui";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import {
  completionSoundEnabled,
  setCompletionSoundEnabled,
} from "@/shared/lib/completion-sound";
import { useSettingsViewModel } from "../../viewmodel/useSettingsViewModel";
import { SettingsBody, SettingsCard } from "../components/SettingsUi";

const THEMES: Array<{ value: ThemePref; label: string }> = [
  { value: "light", label: "Clair" },
  { value: "dark", label: "Sombre" },
  { value: "system", label: "Système" },
];

const SONS = [
  { value: "on", label: "Activé" },
  { value: "off", label: "Désactivé" },
] as const;

/** Thème et retours sonores de l'application. */
export function CustomisationPage() {
  const vm = useSettingsViewModel();
  const setTheme = useUiStore((state) => state.setTheme);
  const [draft, setDraft] = useState<Settings | null>(null);
  const [son, setSon] = useState<"on" | "off">(() =>
    completionSoundEnabled() ? "on" : "off",
  );
  const form = draft ?? vm.data ?? null;

  const save = async () => {
    if (!form) return;
    await vm.save(form, null);
    setDraft(null);
  };

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="palette"
        title="Customisation"
        subtitle="Apparence et retours sonores"
        primary={
          <Button
            variant="primary"
            icon={vm.isSaving ? "progress_activity" : "save"}
            disabled={!form || vm.isSaving}
            onClick={() => void save()}
          >
            {vm.isSaving ? "Enregistrement…" : "Enregistrer"}
          </Button>
        }
      />

      {vm.error && !vm.data ? (
        <div className="px-[18px] pt-4">
          <ErrorBanner
            message={vm.error instanceof AppError ? vm.error.message : "Les réglages n'ont pas pu être chargés."}
            onRetry={vm.recharger}
          />
        </div>
      ) : vm.isLoading || !form ? (
        <div
          className="max-w-[1000px] space-y-4 px-[18px] pt-4"
          role="status"
          aria-label="Chargement des réglages"
        >
          <Skeleton className="h-60 w-full rounded-card" />
        </div>
      ) : (
        <SettingsBody>
          <SettingsCard icon="palette" title="Apparence et son">
            <div className="flex flex-wrap gap-x-8 gap-y-4">
              <div>
                <p className="mb-1.5 text-label font-mid text-ink-muted">Thème</p>
                <SegmentedControl
                  label="Thème"
                  value={form.theme}
                  onChange={(theme) => {
                    setDraft({ ...form, theme });
                    setTheme(theme);
                    applyTheme(theme);
                  }}
                  options={THEMES}
                />
              </div>
              <div>
                <p className="mb-1.5 text-label font-mid text-ink-muted">Son de fin de traitement</p>
                <SegmentedControl
                  label="Son de fin de traitement"
                  value={son}
                  onChange={(valeur) => {
                    setSon(valeur);
                    setCompletionSoundEnabled(valeur === "on");
                  }}
                  options={SONS}
                />
              </div>
            </div>
          </SettingsCard>
        </SettingsBody>
      )}
    </div>
  );
}
