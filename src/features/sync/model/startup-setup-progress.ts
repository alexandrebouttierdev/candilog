export type StartupStorageChoice = "local" | "cloud";

export interface StartupSetupProgress {
  storage: StartupStorageChoice;
  aiPromptDismissed: boolean;
}

const STORAGE_KEY = "candilog.startup-setup";

function isStartupSetupProgress(value: unknown): value is StartupSetupProgress {
  return (
    typeof value === "object" &&
    value !== null &&
    "storage" in value &&
    (value.storage === "local" || value.storage === "cloud") &&
    "aiPromptDismissed" in value &&
    typeof value.aiPromptDismissed === "boolean"
  );
}

export function readStartupSetupProgress(): StartupSetupProgress | null {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    return isStartupSetupProgress(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function saveStartupSetupProgress(progress: StartupSetupProgress): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(progress));
  } catch {
    // Le parcours reste utilisable pendant cette session si le stockage est indisponible.
  }
}
