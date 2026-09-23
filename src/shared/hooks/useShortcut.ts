import { useEffect, useRef } from "react";
import { hasOpenSurface } from "./useDismissable";

/** Champs `<input>` qui ne reçoivent pas de texte : un raccourci à une lettre y reste actif. */
const NON_TEXT_INPUTS = new Set(["checkbox", "radio", "range", "button", "submit", "reset", "color", "file"]);

/**
 * La frappe vise-t-elle un champ ? Un raccourci à une lettre ne doit jamais voler la
 * saisie : `N` tapé dans un champ reste un « n ».
 */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    (target instanceof HTMLInputElement && !NON_TEXT_INPUTS.has(target.type)) ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target.isContentEditable === true
  );
}

/**
 * Raccourci clavier d'écran, en notation neutre : `n`, `shift+n`, `mod+d`, `mod+backspace`.
 *
 * Désactivé quand le focus est dans un champ ou qu'une surface (modale, menu, palette) est
 * ouverte, sauf `global` (réservé à `⌘K` et `⌘,`, actifs partout).
 */
export function useShortcut(
  shortcut: string,
  handler: (event: KeyboardEvent) => void,
  { enabled = true, global = false }: { enabled?: boolean; global?: boolean } = {},
) {
  const handlerRef = useRef(handler);
  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return;
    const listener = (event: KeyboardEvent) => {
      if (!matches(shortcut, event)) return;
      if (!global && (isTypingTarget(event.target) || hasOpenSurface())) return;
      event.preventDefault();
      handlerRef.current(event);
    };
    document.addEventListener("keydown", listener);
    return () => document.removeEventListener("keydown", listener);
  }, [shortcut, enabled, global]);
}

/** L'événement correspond-il au raccourci ? `mod` = ⌘ sous macOS, Ctrl ailleurs. */
export function matches(shortcut: string, event: KeyboardEvent): boolean {
  const parts = shortcut.toLowerCase().split("+");
  const key = parts[parts.length - 1]!;
  const mod = parts.includes("mod");
  const shift = parts.includes("shift");
  const alt = parts.includes("alt");
  if (mod !== (event.metaKey || event.ctrlKey)) return false;
  if (shift !== event.shiftKey || alt !== event.altKey) return false;
  const pressed = event.key.toLowerCase();
  if (key === "enter") return pressed === "enter";
  if (key === "backspace") return pressed === "backspace" || pressed === "delete";
  return pressed === key;
}
