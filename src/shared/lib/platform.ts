/**
 * Plateforme d'exécution et notation des raccourcis clavier.
 *
 * Le design imprime chaque raccourci dans l'interface (barre d'état, menus, boutons) selon
 * l'usage de la plateforme (`reference_design/PLATFORM.md` C4) : glyphes collés sous macOS
 * (`⌘K`), touches écrites en entier et séparées par une espace sous Windows et Linux
 * (`Ctrl K`) — jamais de `+`, jamais `Ctl` ni `^`.
 */

export type Platform = "mac" | "other";

/** Plateforme courante, déduite de l'agent utilisateur de la webview. */
export function currentPlatform(): Platform {
  if (typeof navigator === "undefined") return "other";
  return /Mac|iPhone|iPad/i.test(navigator.userAgent) ? "mac" : "other";
}

/**
 * Touche de raccourci, dans une notation neutre : `mod` (⌘ ou Ctrl), `shift`, `alt`,
 * `enter`, `backspace`, `escape`, ou un caractère (`k`, `,`, `.`).
 */
export type ShortcutKey = string;

const MAC: Record<string, string> = {
  mod: "⌘",
  shift: "⇧",
  alt: "⌥",
  enter: "⏎",
  backspace: "⌫",
  escape: "Échap",
  up: "↑",
  down: "↓",
};

const OTHER: Record<string, string> = {
  mod: "Ctrl",
  shift: "Maj",
  alt: "Alt",
  enter: "⏎",
  backspace: "Suppr",
  escape: "Échap",
  up: "↑",
  down: "↓",
};

/**
 * Notation imprimée d'un raccourci, par exemple `formatShortcut("mod+k")` → `⌘K` sous
 * macOS et `Ctrl K` ailleurs.
 */
export function formatShortcut(shortcut: string, platform: Platform = currentPlatform()): string {
  const table = platform === "mac" ? MAC : OTHER;
  const parts = shortcut
    .split("+")
    .filter((part) => part.length > 0)
    .map((part) => table[part.toLowerCase()] ?? part.toUpperCase());
  return platform === "mac" ? parts.join("") : parts.join(" ");
}
