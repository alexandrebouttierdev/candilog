import { create } from "zustand";

/** Préférence de thème, alignée sur l'enum `ThemePref` du backend. */
export type ThemePref = "light" | "dark" | "system";

export interface ToastMessage {
  readonly id: string;
  readonly tone: "success" | "error" | "info";
  readonly title: string;
  readonly detail?: string | undefined;
}

/**
 * Sections de la surcouche Réglages (`reference_design/INTERACTIONS.md` §3.7). Les
 * Réglages ne sont pas une destination : les fermer rend l'écran précédent intact.
 */
export type SettingsSection = "appearance" | "data" | "ai" | "shortcuts" | "updates" | "about";

interface UiState {
  theme: ThemePref;
  /** Surcouche Réglages ouverte, et sa section. */
  settings: SettingsSection | null;
  /** Palette de commandes `⌘K` ouverte. */
  palette: boolean;
  openSettings: (section?: SettingsSection) => void;
  closeSettings: () => void;
  setPalette: (open: boolean) => void;
  toasts: ToastMessage[];
  setTheme: (theme: ThemePref) => void;
  notify: (toast: Omit<ToastMessage, "id">) => void;
  dismissToast: (id: string) => void;
}

/**
 * État global d'interface.
 *
 * Zustand n'accueille que ce qui est **transverse et non serveur** (docs/CODE_RULES.md §4) :
 * ici, la préférence de thème et la file de notifications. Les données métier restent dans
 * TanStack Query, qui sait déjà les mettre en cache et les invalider ; les dupliquer ici
 * créerait deux vérités à resynchroniser.
 *
 * La préférence de thème est *aussi* persistée en base (table `parametres`) : ce store en
 * est le reflet immédiat pour l'affichage, la persistance passe par la feature Réglages.
 */
export const useUiStore = create<UiState>((set) => ({
  theme: "system",
  toasts: [],
  settings: null,
  palette: false,

  openSettings: (section = "appearance") => set({ settings: section, palette: false }),
  closeSettings: () => set({ settings: null }),
  setPalette: (palette) => set({ palette }),

  setTheme: (theme) => set({ theme }),

  // Un seul toast à la fois (décision E15 du design) : le nouveau remplace le précédent
  // immédiatement, sans file ni pile. Deux actions rapides n'empilent jamais deux bandeaux.
  notify: (toast) => set({ toasts: [{ ...toast, id: crypto.randomUUID() }] }),

  dismissToast: (id) =>
    set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) })),
}));

/**
 * Applique la préférence au document.
 *
 * `system` **retire** l'attribut au lieu d'y écrire une valeur : les feuilles de style font
 * alors jouer `prefers-color-scheme`, et le thème suit l'OS en direct sans que rien n'ait à
 * écouter le changement.
 */
export function applyTheme(theme: ThemePref): void {
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

/** Thème sombre effectif (préférence explicite ou `prefers-color-scheme`). */
export function themeIsDark(theme: ThemePref): boolean {
  return (
    theme === "dark" ||
    (theme === "system" &&
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches)
  );
}

/** Coque rail + topbar en bleu Candilog (mode clair uniquement). */
export function useShellBrand(): boolean {
  const theme = useUiStore((state) => state.theme);
  return !themeIsDark(theme);
}
