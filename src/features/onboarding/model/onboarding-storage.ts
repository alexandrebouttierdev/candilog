const STORAGE_KEY = "candilog.onboarding-termine";

/**
 * Le tour d'accueil a-t-il déjà été vu ?
 *
 * Comme la préférence du son de fin de traitement (`completion-sound.ts`), c'est un état
 * d'interface pur — il ne décrit ni la recherche d'emploi ni le profil — et n'a donc rien à
 * faire dans la base (docs/DATA.md). `localStorage` suffit et survit au redémarrage.
 *
 * Contrairement au son, un stockage indisponible retombe ici sur « déjà vu » plutôt que sur
 * le comportement par défaut : le tour est bloquant, et le rouvrir à chaque lancement parce
 * que le moteur refuse `localStorage` piégerait l'utilisateur au lieu de l'aider.
 */
export function onboardingCompleted(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === "1";
  } catch {
    return true;
  }
}

/** Marque le tour comme vu ; il ne se rouvrira plus au démarrage. */
export function markOnboardingCompleted(): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, "1");
  } catch {
    // Stockage refusé : le tour se rouvrira à la prochaine session, sans autre conséquence.
  }
}

/**
 * Remet le tour à l'état « jamais vu ».
 *
 * Appelé après une réinitialisation complète des données : l'application redevient neuve,
 * la présentation doit donc se rejouer comme au premier lancement.
 */
export function resetOnboarding(): void {
  try {
    window.localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Stockage refusé : le tour ne se rouvrira pas, sans autre conséquence.
  }
}
