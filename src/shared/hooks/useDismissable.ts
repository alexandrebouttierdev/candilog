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

export function useDismissable({
  open,
  onDismiss,
  onSubmit,
}: {
  open: boolean;
  onDismiss: () => void;
  onSubmit?: () => void;
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
        onDismiss();
        return;
      }
      if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && onSubmit) {
        event.preventDefault();
        onSubmit();
      }
    };

    document.addEventListener("keydown", handler, true);
    return () => {
      document.removeEventListener("keydown", handler, true);
      const i = stack.indexOf(token);
      if (i >= 0) stack.splice(i, 1);
    };
  }, [open, onDismiss, onSubmit]);
}
