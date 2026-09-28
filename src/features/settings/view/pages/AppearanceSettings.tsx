import { useState } from "react";
import type { ThemePref } from "@/shared/types/generated/settings";
import { SegmentedControl, Switch } from "@/shared/ui";
import { useUiStore } from "@/shared/lib/ui-store";
import { completionSoundEnabled, setCompletionSoundEnabled } from "@/shared/lib/completion-sound";
import {
  animationsEnabled,
  density,
  setAnimationsEnabled,
  setDensity,
  type Density,
} from "@/shared/lib/display-prefs";
import { useThemePreference } from "../../viewmodel/useSettingsViewModel";
import { SettingsRow, SettingsSection } from "../components/SettingsSection";

const THEMES: ReadonlyArray<{ value: ThemePref; label: string }> = [
  { value: "light", label: "Clair" },
  { value: "dark", label: "Sombre" },
  { value: "system", label: "Système" },
];

const DENSITES: ReadonlyArray<{ value: Density; label: string }> = [
  { value: "compact", label: "Compacte" },
  { value: "comfortable", label: "Confortable" },
];

/**
 * Réglages → Apparence. Tout s'applique immédiatement, sans bouton « Enregistrer » : le
 * thème est aussi enregistré en base (il suit l'utilisateur d'une restauration à l'autre),
 * densité, animations et son restent propres à cet ordinateur.
 */
export function AppearanceSettings() {
  const theme = useUiStore((state) => state.theme);
  const { saveTheme } = useThemePreference();
  const [densite, setDensiteState] = useState<Density>(density);
  const [animations, setAnimationsState] = useState(animationsEnabled);
  const [son, setSon] = useState(completionSoundEnabled);

  return (
    <SettingsSection
      title="Apparence"
      description="Comment Candilog s'affiche sur cet ordinateur. Ces réglages ne changent rien à vos documents."
    >
      <SettingsRow label="Thème" hint="« Système » suit le réglage de votre ordinateur.">
        <SegmentedControl
          label="Thème"
          value={theme}
          options={THEMES}
          onChange={(next) => void saveTheme(next)}
        />
      </SettingsRow>
      <SettingsRow label="Densité des listes" hint="Nombre de lignes visibles sans faire défiler.">
        <SegmentedControl
          label="Densité des listes"
          value={densite}
          options={DENSITES}
          onChange={(next) => {
            setDensity(next);
            setDensiteState(next);
          }}
        />
      </SettingsRow>
      <SettingsRow label="Animations" hint="Désactivez-les si l'affichage saccade ou vous gêne.">
        <Switch
          label="Animations"
          checked={animations}
          onChange={(next) => {
            setAnimationsEnabled(next);
            setAnimationsState(next);
          }}
        />
      </SettingsRow>
      <SettingsRow
        label="Son de fin de traitement"
        hint="Deux notes brèves quand une génération ou une analyse se termine."
      >
        <Switch
          label="Son de fin de traitement"
          checked={son}
          onChange={(next) => {
            setCompletionSoundEnabled(next);
            setSon(next);
          }}
        />
      </SettingsRow>
      <SettingsRow label="Langue">
        <span className="text-ui text-tx-3">Français</span>
      </SettingsRow>
    </SettingsSection>
  );
}
