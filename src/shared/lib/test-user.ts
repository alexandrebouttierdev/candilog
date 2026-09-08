import userEvent from "@testing-library/user-event";

/**
 * Utilisateur simulé qui frappe sans délai entre les touches.
 *
 * Le défaut de `user-event` rend la main entre deux caractères. Sur une machine chargée,
 * React re-rend le champ dans cet intervalle : une date saisie « 01-08-2026 » ressortait
 * « 0028-12-20 », et un poste tapé après un autre champ héritait de ses dernières lettres
 * (« talDéveloppeur Frontend »). Sans délai, la séquence de touches n'est plus
 * entrecoupée, et le test mesure le comportement du composant plutôt que la charge de la
 * machine.
 *
 * À réserver aux tests qui saisissent du texte : un clic simple n'a pas ce problème.
 */
export function userWithoutDelay() {
  return userEvent.setup({ delay: null });
}
