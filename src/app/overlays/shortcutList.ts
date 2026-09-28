/** Un raccourci documenté dans Réglages → Raccourcis, en notation neutre. */
export interface ShortcutDoc {
  readonly keys: readonly string[];
  readonly effect: string;
  readonly context: string;
}

/**
 * Raccourcis **réellement câblés** à ce stade de la refonte. La liste s'allonge avec les
 * écrans : n'y inscrire qu'un raccourci qui fonctionne, jamais une promesse.
 */
export const SHORTCUTS: readonly ShortcutDoc[] = [
  { keys: ["mod+k"], effect: "Palette de commandes", context: "partout" },
  { keys: ["mod+,"], effect: "Réglages", context: "partout" },
  { keys: ["escape"], effect: "Fermer le niveau le plus haut", context: "partout" },
  { keys: ["g", "a"], effect: "Aller à Aujourd'hui", context: "hors saisie" },
  { keys: ["g", "c"], effect: "Aller aux Candidatures", context: "hors saisie" },
  { keys: ["g", "r"], effect: "Aller aux Relations", context: "hors saisie" },
  { keys: ["g", "d"], effect: "Aller aux Documents", context: "hors saisie" },
  { keys: ["g", "p"], effect: "Aller au Profil", context: "hors saisie" },
  { keys: ["mod+enter"], effect: "Valider un formulaire", context: "formulaire" },
];
