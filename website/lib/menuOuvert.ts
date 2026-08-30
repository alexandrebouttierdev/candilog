/**
 * Registre du menu de téléchargement actuellement ouvert.
 *
 * Il y a deux instances de `DownloadMenu` sur la landing (hero et bloc de
 * téléchargement) et le §7.11 impose qu'elles ne soient jamais ouvertes en même
 * temps. Un store module plutôt qu'un contexte : les deux instances vivent dans des
 * sections différentes, et ça évite de faire remonter un provider jusqu'au layout.
 */
let ouvert: string | null = null;
const abonnes = new Set<() => void>();

export function ouvrirMenu(id: string | null) {
  if (ouvert === id) return;
  ouvert = id;
  for (const notifier of abonnes) notifier();
}

export function abonnerMenu(onStoreChange: () => void) {
  abonnes.add(onStoreChange);
  return () => {
    abonnes.delete(onStoreChange);
  };
}

export const lireMenuOuvert = () => ouvert;
export const lireMenuOuvertServeur = () => null;
