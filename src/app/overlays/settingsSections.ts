import type { SettingsSection } from "@/shared/lib/ui-store";

/** Libellés des six sections de la surcouche Réglages, dans l'ordre du design. */
export const SETTINGS_LABELS: Readonly<Record<SettingsSection, string>> = {
  appearance: "Apparence",
  data: "Synchronisation et sauvegarde",
  ai: "Intelligence artificielle",
  shortcuts: "Raccourcis",
  updates: "Mises à jour",
  about: "À propos",
};

export const SETTINGS_ORDER: readonly SettingsSection[] = [
  "appearance",
  "data",
  "ai",
  "shortcuts",
  "updates",
  "about",
];
