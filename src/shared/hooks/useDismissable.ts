import { useEffect } from "react";

/**
 * Ferme une surface superposée sur Échap et valide sur Ctrl/Cmd+Entrée.
 *
 * Le guide impose ces deux raccourcis sur toutes les modales. Les câbler dans un hook plutôt
 * que dans chaque formulaire évite qu'ils manquent sur celles écrites plus tard, et garantit
 * que seule la surface la plus haute réagit : l'écouteur est posé en phase de capture sur le
 * document, et chaque surface ouverte s'enregistre dans une pile.
 */
const stack: symbol[] = [];

/**
 * Une surface superposée (modale, dialogue, menu, palette) est-elle ouverte ? Les
 * raccourcis d'écran à une lettre (`N`, `R`, `G`…) se taisent alors : la touche appartient
 * à la surface la plus haute.
 */
export function hasOpenSurface(): boolean {
  return stack.length > 0;
}

export function useDismissable({
  open,
  onDismiss,
  onSubmit,
  onEnter,
  dismissDisabled = false,
}: {
  open: boolean;
  onDismiss: () => void;
  onSubmit?: () => void;
  /**
   * `⏎` seul confirme (contrat clavier des dialogues du design). Ignoré quand le focus est
   * sur un bouton, un lien ou une zone de texte : leur comportement natif prime — `⏎` sur
   * « Annuler » annule, et `⏎` dans une zone de texte reste un retour à la ligne.
   */
  onEnter?: () => void;
  dismissDisabled?: boolean;
}) {
  useEffect(() => {
    if (!open) return;

    const token = Symbol("surface");
    stack.push(token);

    const handler = (event: KeyboardEvent) => {
      // Seule la surface au sommet réagit : deux modales empilées ne doivent pas se fermer
      // ensemble sur un seul Échap.
      if (stack[stack.length - 1] !== token) return;

      if (event.key === "Escape") {
        event.stopPropagation();
        if (!dismissDisabled) onDismiss();
        return;
      }
      if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && onSubmit) {
        event.preventDefault();
        onSubmit();
        return;
      }
      if (
        event.key === "Enter" &&
        !event.metaKey &&
        !event.ctrlKey &&
        !event.shiftKey &&
        !event.altKey &&
        onEnter &&
        !enterBelongsToFocusedControl(document.activeElement)
      ) {
        event.preventDefault();
        onEnter();
      }
    };

    document.addEventListener("keydown", handler, true);
    return () => {
      document.removeEventListener("keydown", handler, true);
      const i = stack.indexOf(token);
      if (i >= 0) stack.splice(i, 1);
    };
  }, [dismissDisabled, open, onDismiss, onSubmit, onEnter]);
}

/** `⏎` a déjà un sens natif sur ce contrôle : bouton, lien, zone de texte, contenu éditable. */
function enterBelongsToFocusedControl(element: Element | null): boolean {
  if (!(element instanceof HTMLElement)) return false;
  return (
    element instanceof HTMLButtonElement ||
    element instanceof HTMLAnchorElement ||
    element instanceof HTMLTextAreaElement ||
    element.isContentEditable
  );
}
