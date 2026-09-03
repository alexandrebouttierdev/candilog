const STORAGE_KEY = "candilog.son-fin-traitement";

/**
 * Signal sonore de fin de traitement IA.
 *
 * La préférence reste **locale** : elle ne décrit ni la recherche d'emploi ni le profil, et
 * n'a donc rien à faire dans la base (docs/DATA.md). `localStorage` suffit et survit au
 * redémarrage. Le son est actif par défaut : une génération dure assez longtemps pour que
 * l'utilisateur aille voir ailleurs.
 */
export function completionSoundEnabled(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) !== "off";
  } catch {
    // Stockage refusé par le moteur : on garde le comportement par défaut.
    return true;
  }
}

/** Enregistre la préférence ; l'appel suivant à `playCompletionSound` en tient compte. */
export function setCompletionSoundEnabled(enabled: boolean): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, enabled ? "on" : "off");
  } catch {
    // Stockage refusé : la préférence ne survivra pas à la session, sans autre conséquence.
  }
}

/**
 * Deux notes brèves, synthétisées à la volée.
 *
 * Aucun fichier audio n'est embarqué : le son est produit par l'API Web Audio, ce qui évite
 * un asset de plus dans le binaire et reste inaudible pour les tests, où `AudioContext`
 * n'existe pas.
 *
 * L'appel arrive toujours après un `await` (la génération vient de se résoudre), donc hors
 * du geste utilisateur qui a déclenché le traitement. La politique de lecture automatique du
 * moteur crée alors le contexte à l'état `suspended` : sans le `resume()` explicite ci-dessous,
 * les oscillateurs sont bien programmés mais aucun son n'est audible, le silence n'étant
 * signalé par aucune erreur.
 */
export function playCompletionSound(): void {
  if (!completionSoundEnabled()) return;
  const Constructeur = window.AudioContext;
  if (!Constructeur) return;
  try {
    const context = new Constructeur();
    if (context.state === "suspended") {
      context.resume().catch(() => {
        // Reprise refusée par le moteur : le silence reste préférable à une exception.
      });
    }
    note(context, 880, context.currentTime);
    note(context, 1174.66, context.currentTime + 0.11);
    window.setTimeout(() => void context.close(), 700);
  } catch {
    // Sortie audio indisponible : le traitement s'est terminé, le son est accessoire.
  }
}

function note(context: AudioContext, frequency: number, at: number): void {
  const oscillator = context.createOscillator();
  const gain = context.createGain();
  oscillator.type = "sine";
  oscillator.frequency.setValueAtTime(frequency, at);
  gain.gain.setValueAtTime(0.0001, at);
  gain.gain.exponentialRampToValueAtTime(0.14, at + 0.02);
  gain.gain.exponentialRampToValueAtTime(0.0001, at + 0.16);
  oscillator.connect(gain);
  gain.connect(context.destination);
  oscillator.start(at);
  oscillator.stop(at + 0.18);
}
