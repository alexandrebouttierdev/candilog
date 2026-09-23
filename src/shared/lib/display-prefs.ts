/**
 * Préférences d'affichage propres à cet ordinateur : densité des listes et animations.
 *
 * Comme le son de fin de traitement, elles ne décrivent ni la recherche d'emploi ni le
 * profil : elles restent dans `localStorage`, hors de la base (docs/DATA.md). Elles
 * s'appliquent par des attributs sur `<html>`, que `styles.css` traduit en jetons :
 * `data-density="compact"` réduit les hauteurs de ligne (`DECISIONS.md` E9 du design),
 * `data-motion="off"` ramène les durées d'animation à 0,01 ms, comme la réduction de
 * mouvement du système.
 */

export type Density = "comfortable" | "compact";

const DENSITY_KEY = "candilog.densite";
const MOTION_KEY = "candilog.animations";

function read(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    // Stockage refusé par le moteur : on garde les valeurs par défaut.
    return null;
  }
}

function write(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // Stockage refusé : la préférence ne survivra pas à la session, sans autre conséquence.
  }
}

/** Densité enregistrée ; « Confortable » par défaut. */
export function density(): Density {
  return read(DENSITY_KEY) === "compact" ? "compact" : "comfortable";
}

/** Animations actives ? Oui par défaut ; la réduction de mouvement du système prime en CSS. */
export function animationsEnabled(): boolean {
  return read(MOTION_KEY) !== "off";
}

export function setDensity(value: Density): void {
  write(DENSITY_KEY, value);
  applyDisplayPrefs();
}

export function setAnimationsEnabled(enabled: boolean): void {
  write(MOTION_KEY, enabled ? "on" : "off");
  applyDisplayPrefs();
}

/** Reflète les préférences enregistrées sur `<html>`. */
export function applyDisplayPrefs(): void {
  const root = document.documentElement;
  if (density() === "compact") root.setAttribute("data-density", "compact");
  else root.removeAttribute("data-density");
  if (animationsEnabled()) root.removeAttribute("data-motion");
  else root.setAttribute("data-motion", "off");
}
